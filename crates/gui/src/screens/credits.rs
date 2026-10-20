use egui::{
    Align2, Color32, FontId, Id, Key, LayerId, Order, Pos2, Rect, RichText, Stroke, Vec2,
};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Action emitted by the End credits screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditsAction {
    None,
    CloseAndRespawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditLineStyle {
    Title,
    Subtitle,
    SpeakerGreen,
    SpeakerAqua,
    Narrative,
    Header,
    CreditRole,
    CreditName,
    Empty,
}

#[derive(Debug, Clone)]
pub struct CreditLine {
    pub text: String,
    pub style: CreditLineStyle,
}

/// The Minecraft 1.8.9 End Poem and Credits Screen (GuiWinGame).
pub struct CreditsScreen {
    pub lines: Vec<CreditLine>,
    pub scroll_y: f32,
    pub total_height: f32,
    pub player_name: String,
    last_frame_time: Option<Instant>,
}

impl CreditsScreen {
    pub fn new(player_name: &str) -> Self {
        let mut screen = Self {
            lines: Vec::new(),
            scroll_y: 0.0,
            total_height: 0.0,
            player_name: player_name.to_string(),
            last_frame_time: None,
        };
        screen.load_credits();
        screen
    }

    /// Loads End Poem and Credits from assets if available, or falls back to authentic built-in texts.
    fn load_credits(&mut self) {
        self.lines.clear();

        // Title banner
        self.lines.push(CreditLine {
            text: "MINECRAFT".to_string(),
            style: CreditLineStyle::Title,
        });
        self.lines.push(CreditLine {
            text: "The End Poem".to_string(),
            style: CreditLineStyle::Subtitle,
        });
        self.lines.push(CreditLine {
            text: String::new(),
            style: CreditLineStyle::Empty,
        });
        self.lines.push(CreditLine {
            text: String::new(),
            style: CreditLineStyle::Empty,
        });

        let mut loaded_from_file = false;
        let end_txt_path = Path::new("assets/minecraft/texts/end.txt");
        let credits_txt_path = Path::new("assets/minecraft/texts/credits.txt");

        if end_txt_path.exists() {
            if let Ok(content) = fs::read_to_string(end_txt_path) {
                loaded_from_file = true;
                let mut is_speaker_green = true;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        self.lines.push(CreditLine {
                            text: String::new(),
                            style: CreditLineStyle::Empty,
                        });
                        continue;
                    }
                    let replaced = trimmed.replace("PLAYERNAME", &self.player_name);
                    let style = if is_speaker_green {
                        CreditLineStyle::SpeakerGreen
                    } else {
                        CreditLineStyle::SpeakerAqua
                    };
                    is_speaker_green = !is_speaker_green;
                    self.lines.push(CreditLine {
                        text: replaced,
                        style,
                    });
                }
            }
        }

        if credits_txt_path.exists() {
            if let Ok(content) = fs::read_to_string(credits_txt_path) {
                self.lines.push(CreditLine {
                    text: String::new(),
                    style: CreditLineStyle::Empty,
                });
                self.lines.push(CreditLine {
                    text: "CREDITS".to_string(),
                    style: CreditLineStyle::Header,
                });
                self.lines.push(CreditLine {
                    text: String::new(),
                    style: CreditLineStyle::Empty,
                });
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        self.lines.push(CreditLine {
                            text: String::new(),
                            style: CreditLineStyle::Empty,
                        });
                    } else if trimmed.starts_with("[C]") {
                        self.lines.push(CreditLine {
                            text: trimmed.trim_start_matches("[C]").trim().to_string(),
                            style: CreditLineStyle::Header,
                        });
                    } else if trimmed.contains(':') {
                        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                        self.lines.push(CreditLine {
                            text: parts[0].trim().to_string(),
                            style: CreditLineStyle::CreditRole,
                        });
                        self.lines.push(CreditLine {
                            text: parts[1].trim().to_string(),
                            style: CreditLineStyle::CreditName,
                        });
                    } else {
                        self.lines.push(CreditLine {
                            text: trimmed.to_string(),
                            style: CreditLineStyle::Narrative,
                        });
                    }
                }
            }
        }

        // Fallback to complete authentic End Poem by Julian Gough if file was not on disk
        if !loaded_from_file {
            self.load_default_poem();
        }

        // Compute total scroll height
        let mut total = 0.0;
        for line in &self.lines {
            total += match line.style {
                CreditLineStyle::Title => 44.0,
                CreditLineStyle::Subtitle => 26.0,
                CreditLineStyle::Header => 32.0,
                CreditLineStyle::Empty => 16.0,
                CreditLineStyle::SpeakerGreen | CreditLineStyle::SpeakerAqua => 28.0,
                CreditLineStyle::CreditRole => 20.0,
                CreditLineStyle::CreditName => 22.0,
                CreditLineStyle::Narrative => 22.0,
            };
        }
        self.total_height = total;
    }

    fn load_default_poem(&mut self) {
        let p_name = self.player_name.clone();
        let poem_lines = [
            (true, "I see the player you mean."),
            (false, &format!("{}?", p_name)),
            (true, "Yes. Take care. It has reached a higher level now. It can read our thoughts."),
            (false, "That does not matter. It thinks we are part of the game."),
            (true, "I like this player. It played well. It did not give up."),
            (false, "It is reading our thoughts as though they were words on a screen."),
            (true, "That is how it chooses to imagine many things, when it is deep in the dream of a game."),
            (false, "Words make a wonderful interface. Very flexible. And less terrifying than the reality behind the screen."),
            (true, "They used to hear voices. Before players could read. Back in the days when those who did not play called the players witches, and warlocks."),
            (false, "What did this player dream?"),
            (true, "This player dreamed of sunlight and trees. Of fire and water. It dreamed it created. And it dreamed it destroyed. It dreamed it hunted, and was hunted. It dreamed of shelter."),
            (false, "Hah, the original interface. A million years old, and it still works. But what was the real structure behind this player's screen, in the truth of the universe?"),
            (true, "It worked, with a million others, to sculpt a true world in a fold of the cosmos."),
            (false, "It cannot read that thought."),
            (true, "No. It has not yet achieved the highest level. That, it must achieve in the long dream of life, not the short dream of a game."),
            (false, "Does it know that we love it? That the universe is kind?"),
            (true, "Sometimes, through the noise of its thoughts, it hears the universe, yes."),
            (false, "But there are times it is sad, in the long dream. It creates worlds that have no summer, and it shivers under a black sun."),
            (true, "To cure it of sorrow would destroy it. The sorrow is part of its own private task. We cannot interfere."),
            (false, "Sometimes when they are deep in dreams, I want to tell them, they are building true worlds in reality."),
            (true, "It reads our thoughts."),
            (false, "Sometimes I do not care. Sometimes I wish to tell them, this world you take for truth is merely a dream."),
            (true, "And yet they play the game."),
            (false, "I will not tell the player how to live."),
            (true, "The player is growing restless."),
            (false, "I will tell the player a story."),
            (true, "A story that contains the truth, in words it can comprehend."),
            (false, "Give it a body, again."),
            (true, "Yes. The player..."),
            (false, "Use its name."),
            (true, &format!("{}. Player of games.", p_name)),
            (false, "Good."),
            (true, "Take a breath, now. Take another. Feel air in your lungs. Let your limbs return. Have a body again, under gravity, in air."),
            (false, "Respawn in the long dream. There you are. Your body touching the universe again at every point, as though you were separate things."),
            (true, "The universe loves you because you are love."),
            (false, "And the game was over and the player woke up from the dream. And the player began a new dream. And the player dreamed again, dreamed better. And the player was the universe. And the player was the love."),
            (true, "You are the player."),
            (false, "Wake up."),
        ];

        for (is_green, text) in poem_lines {
            self.lines.push(CreditLine {
                text: text.to_string(),
                style: if is_green {
                    CreditLineStyle::SpeakerGreen
                } else {
                    CreditLineStyle::SpeakerAqua
                },
            });
            self.lines.push(CreditLine {
                text: String::new(),
                style: CreditLineStyle::Empty,
            });
        }

        self.lines.push(CreditLine {
            text: String::new(),
            style: CreditLineStyle::Empty,
        });
        self.lines.push(CreditLine {
            text: "MINECRAFT CREDITS".to_string(),
            style: CreditLineStyle::Header,
        });
        self.lines.push(CreditLine {
            text: String::new(),
            style: CreditLineStyle::Empty,
        });

        let credits = [
            ("Original Creator", "Markus Persson (Notch)"),
            ("Lead Developer", "Jens Bergensten (Jeb)"),
            ("Music & Sound Effects", "Daniel Rosenfeld (C418)"),
            ("Art & Visuals", "Kristoffer Zetterstrand, Markus Toivonen"),
            ("Client Programming", "Notch, Jeb, Dinnerbone, Grum, Searge"),
            ("Client Port (Rust 1.8.9)", "High-Performance Pure Rust Engine"),
        ];

        for (role, name) in credits {
            self.lines.push(CreditLine {
                text: role.to_string(),
                style: CreditLineStyle::CreditRole,
            });
            self.lines.push(CreditLine {
                text: name.to_string(),
                style: CreditLineStyle::CreditName,
            });
            self.lines.push(CreditLine {
                text: String::new(),
                style: CreditLineStyle::Empty,
            });
        }

        self.lines.push(CreditLine {
            text: "Thank you for playing!".to_string(),
            style: CreditLineStyle::Header,
        });
    }

    /// Renders the credits overlay.
    pub fn render(&mut self, ctx: &egui::Context) -> CreditsAction {
        let now = Instant::now();
        let dt = if let Some(last) = self.last_frame_time {
            (now - last).as_secs_f32().min(0.1)
        } else {
            0.016
        };
        self.last_frame_time = Some(now);

        // Escape key skips credits immediately
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            return CreditsAction::CloseAndRespawn;
        }

        // Speed control: Space or Down arrow speeds up (5x)
        let is_speeding_up = ctx.input(|i| i.key_down(Key::Space) || i.key_down(Key::ArrowDown));
        let is_rewinding = ctx.input(|i| i.key_down(Key::ArrowUp));

        let speed = if is_speeding_up {
            220.0
        } else if is_rewinding {
            -100.0
        } else {
            44.0
        };

        self.scroll_y = (self.scroll_y + speed * dt).max(0.0);

        let screen_rect = ctx.screen_rect();
        let width = screen_rect.width();
        let height = screen_rect.height();

        // If credits finished scrolling completely past the screen
        if self.scroll_y > self.total_height + height {
            return CreditsAction::CloseAndRespawn;
        }

        // Fullscreen background: cosmic dark tone
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("credits_backdrop")));
        painter.rect_filled(screen_rect, 0.0, Color32::from_rgb(10, 11, 16));

        // Draw ambient cosmic star specks
        let time_sec = ctx.input(|i| i.time) as f32;
        for i in 0..64 {
            let seed = (i * 1234567 + 98765) as u32;
            let sx = ((seed % 1000) as f32 / 1000.0) * width;
            let sy = (((seed / 1000) % 1000) as f32 / 1000.0) * height;
            let twinkle = ((time_sec * 2.0 + (i as f32)).sin() * 0.5 + 0.5) * 160.0 + 60.0;
            let star_col = Color32::from_rgba_unmultiplied(200, 220, 255, twinkle as u8);
            painter.circle_filled(Pos2::new(sx, sy), 1.2, star_col);
        }

        // Center column for the text
        let content_width = 460.0_f32.min(width - 40.0);
        let center_x = width * 0.5;
        let mut cur_y = height - self.scroll_y;

        for line in &self.lines {
            let line_h = match line.style {
                CreditLineStyle::Title => 44.0,
                CreditLineStyle::Subtitle => 26.0,
                CreditLineStyle::Header => 32.0,
                CreditLineStyle::Empty => 16.0,
                CreditLineStyle::SpeakerGreen | CreditLineStyle::SpeakerAqua => 28.0,
                CreditLineStyle::CreditRole => 20.0,
                CreditLineStyle::CreditName => 22.0,
                CreditLineStyle::Narrative => 22.0,
            };

            // Only render visible lines within screen bounds
            if cur_y + line_h >= -20.0 && cur_y <= height + 20.0 {
                let pos = Pos2::new(center_x, cur_y);
                match line.style {
                    CreditLineStyle::Title => {
                        painter.text(
                            pos,
                            Align2::CENTER_TOP,
                            &line.text,
                            FontId::proportional(28.0),
                            Color32::from_rgb(255, 215, 0),
                        );
                    }
                    CreditLineStyle::Subtitle => {
                        painter.text(
                            pos,
                            Align2::CENTER_TOP,
                            &line.text,
                            FontId::proportional(16.0),
                            Color32::from_rgb(180, 190, 210),
                        );
                    }
                    CreditLineStyle::Header => {
                        painter.text(
                            pos,
                            Align2::CENTER_TOP,
                            &line.text,
                            FontId::proportional(18.0),
                            Color32::from_rgb(255, 220, 100),
                        );
                    }
                    CreditLineStyle::SpeakerGreen => {
                        // Speaker 1 (Green/Lime)
                        let text_rect = Rect::from_center_size(pos + Vec2::new(0.0, 10.0), Vec2::new(content_width, line_h));
                        let galley = painter.layout(
                            line.text.clone(),
                            FontId::proportional(13.5),
                            Color32::from_rgb(85, 255, 85),
                            content_width,
                        );
                        painter.galley(text_rect.left_top(), galley, Color32::from_rgb(85, 255, 85));
                    }
                    CreditLineStyle::SpeakerAqua => {
                        // Speaker 2 (Aqua/Cyan)
                        let text_rect = Rect::from_center_size(pos + Vec2::new(0.0, 10.0), Vec2::new(content_width, line_h));
                        let galley = painter.layout(
                            line.text.clone(),
                            FontId::proportional(13.5),
                            Color32::from_rgb(85, 255, 255),
                            content_width,
                        );
                        painter.galley(text_rect.left_top(), galley, Color32::from_rgb(85, 255, 255));
                    }
                    CreditLineStyle::CreditRole => {
                        painter.text(
                            pos,
                            Align2::CENTER_TOP,
                            &line.text,
                            FontId::proportional(12.5),
                            Color32::from_rgb(160, 170, 185),
                        );
                    }
                    CreditLineStyle::CreditName => {
                        painter.text(
                            pos,
                            Align2::CENTER_TOP,
                            &line.text,
                            FontId::proportional(14.0),
                            Color32::from_rgb(255, 255, 255),
                        );
                    }
                    CreditLineStyle::Narrative => {
                        let text_rect = Rect::from_center_size(pos + Vec2::new(0.0, 10.0), Vec2::new(content_width, line_h));
                        let galley = painter.layout(
                            line.text.clone(),
                            FontId::proportional(13.0),
                            Color32::from_rgb(220, 225, 235),
                            content_width,
                        );
                        painter.galley(text_rect.left_top(), galley, Color32::from_rgb(220, 225, 235));
                    }
                    CreditLineStyle::Empty => {}
                }
            }

            cur_y += line_h;
        }

        // Top-right controls bar and interactive Skip button
        let mut user_action = CreditsAction::None;
        let top_right_rect = Rect::from_min_size(
            Pos2::new(width - 240.0, 16.0),
            Vec2::new(224.0, 32.0),
        );

        egui::Area::new(Id::new("credits_controls_area"))
            .order(Order::Tooltip)
            .fixed_pos(top_right_rect.min)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("[Esc] Skip  [Space] Fast")
                            .size(11.0)
                            .color(Color32::from_rgb(160, 165, 180)),
                    );
                    let skip_btn = egui::Button::new(
                        RichText::new("Skip ▶").size(12.0).color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgba_unmultiplied(40, 44, 56, 220))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 75, 90)));

                    if ui.add(skip_btn).clicked() {
                        user_action = CreditsAction::CloseAndRespawn;
                    }
                });
            });

        // Request continuous repaint for smooth scrolling animation
        ctx.request_repaint();

        user_action
    }
}
