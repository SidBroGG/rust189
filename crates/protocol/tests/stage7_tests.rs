use bytes::BytesMut;
use flate2::read::ZlibDecoder as Flate2ZlibDecoder;
use flate2::write::ZlibEncoder as Flate2ZlibEncoder;
use flate2::Compression as Flate2Compression;
use protocol::codec::varint::{
    decode_varint_fallback, decode_varint_slice, has_bmi2, write_varint,
};
use protocol::transport::compression::{compress_zlib, decompress_zlib};
use protocol::transport::raknet::congestion::CongestionControl;
use protocol::transport::raknet::types::{Frame, Reliability};
use protocol::transport::ring_buffer::ZeroCopyRingBuffer;
use std::io::{Read, Write};
use std::time::Instant;

#[test]
fn test_varint_bmi2_vs_fallback_benchmark_and_parity() {
    let test_values = [
        0, 1, 2, 63, 127, 128, 255, 256, 16383, 16384, 2097151, 2097152,
        134217727, 2147483647, -1, -2, -128, -32768, -2147483648,
    ];

    let mut encoded_stream = BytesMut::new();
    let mut expected_values = Vec::new();

    // Generate 50,000 encoded VarInts
    for i in 0..50000 {
        let val = test_values[i % test_values.len()];
        write_varint(val, &mut encoded_stream);
        expected_values.push(val);
    }

    let slice = encoded_stream.freeze();

    // 1. Benchmark fallback path
    let t0 = Instant::now();
    let mut offset = 0;
    for &expected in &expected_values {
        let (val, len) = decode_varint_fallback(&slice[offset..]).expect("fallback decode failed");
        assert_eq!(val, expected);
        offset += len;
    }
    let elapsed_fallback = t0.elapsed();
    assert_eq!(offset, slice.len());

    // 2. Benchmark slice path (using BMI2 hardware decoder if CPU supports it)
    let t1 = Instant::now();
    offset = 0;
    for &expected in &expected_values {
        let (val, len) = decode_varint_slice(&slice[offset..]).expect("slice decode failed");
        assert_eq!(val, expected);
        offset += len;
    }
    let elapsed_slice = t1.elapsed();
    assert_eq!(offset, slice.len());

    println!(
        "VarInt Decode Benchmark (50,000 ints):\n  Fallback: {:?} ({:.2} ns/op)\n  Accelerated: {:?} ({:.2} ns/op)\n  BMI2 Supported on Host: {}",
        elapsed_fallback,
        (elapsed_fallback.as_nanos() as f64) / 50000.0,
        elapsed_slice,
        (elapsed_slice.as_nanos() as f64) / 50000.0,
        has_bmi2()
    );
}

#[test]
fn test_libdeflate_vs_flate2_parity_and_cross_compatibility() {
    let payloads: Vec<Vec<u8>> = vec![
        b"Small chat message packet \xA7aHello World!".to_vec(),
        vec![0x42; 1024],          // 1 KB uniform payload
        (0..16384).map(|i| (i * 37 % 256) as u8).collect(), // 16 KB pseudorandom payload
        vec![0xAA; 65536],         // 64 KB chunk data simulation
    ];

    for (idx, original) in payloads.iter().enumerate() {
        // A. Compress with libdeflate -> Decompress with flate2 (zlib)
        let compressed_libdeflate = compress_zlib(original).expect("libdeflate compression failed");
        let mut flate2_decoder = Flate2ZlibDecoder::new(&compressed_libdeflate[..]);
        let mut decompressed_flate2 = Vec::new();
        flate2_decoder
            .read_to_end(&mut decompressed_flate2)
            .expect("flate2 decompression failed");
        assert_eq!(
            &decompressed_flate2, original,
            "Mismatch in payload {}: libdeflate -> flate2",
            idx
        );

        // B. Compress with flate2 -> Decompress with libdeflate
        let mut flate2_encoder = Flate2ZlibEncoder::new(Vec::new(), Flate2Compression::fast());
        flate2_encoder.write_all(original).unwrap();
        let compressed_flate2 = flate2_encoder.finish().unwrap();

        let decompressed_libdeflate = decompress_zlib(&compressed_flate2, original.len())
            .expect("libdeflate decompression failed");
        assert_eq!(
            &decompressed_libdeflate, original,
            "Mismatch in payload {}: flate2 -> libdeflate",
            idx
        );
    }
}

#[test]
fn test_zero_copy_ring_buffer_stress() {
    let mut ring = ZeroCopyRingBuffer::with_capacity(8192);

    // Stream 10,000 variable-sized packets through the ring buffer
    for i in 0..10000 {
        let size = (i % 250) + 1;
        let payload: Vec<u8> = (0..size).map(|b| ((b + i) % 256) as u8).collect();

        // Write directly into ring buffer
        let slice = ring.prepare_write_slice(size);
        slice[..size].copy_from_slice(&payload);
        ring.commit_write(size);

        // Read and verify immediately or in batches
        if i % 3 == 0 || ring.readable_len() > 4096 {
            let readable = ring.readable_slice();
            assert!(readable.len() >= size);
            assert_eq!(&readable[readable.len() - size..], &payload[..]);
            ring.advance_read(readable.len());
        }
    }

    if !ring.is_empty() {
        ring.advance_read(ring.readable_len());
    }
    assert!(ring.is_empty());
}

#[test]
fn test_raknet_frames_and_congestion_control() {
    // 1. Frame serialization & roundtrip
    let payload = b"Test RakNet payload packet data".to_vec();
    let frame = Frame::new_reliable_ordered(payload.clone(), 42, 10, 0);

    let mut buf = BytesMut::new();
    frame.encode(&mut buf);

    let mut reader = buf.freeze();
    let decoded = Frame::decode(&mut reader).expect("Failed to decode RakNet frame");
    assert_eq!(decoded.reliability, Reliability::ReliableOrdered);
    assert_eq!(decoded.reliable_index, Some(42));
    assert_eq!(decoded.order_index, Some(10));
    assert_eq!(decoded.body, payload);

    // 2. Congestion control transitions
    let mut cc = CongestionControl::new(1464);
    assert!(cc.can_send(1000));

    // Send MTU-sized packet
    cc.on_packet_sent(1464);
    assert_eq!(cc.bytes_in_flight, 1464);

    // ACK received in slow start
    let old_cwnd = cc.cwnd;
    cc.on_ack_received(1464, Some(25.0));
    assert!(cc.cwnd > old_cwnd);
    assert_eq!(cc.bytes_in_flight, 0);

    // Simulate packet loss
    cc.on_packet_loss(1464);
    assert_eq!(cc.total_packets_lost, 1);
    assert!(cc.cwnd <= cc.ssthresh);
}
