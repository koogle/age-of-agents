//! Resource totals, selected commands and their responsive placement.
use super::*;

impl Hud {
    /// Lays out the whole interface for this frame.
    pub fn layout(&mut self, atlas: &Atlas, model: &Model, width: f32, height: f32, scale: f32) {
        self.quads.clear();
        self.regions.clear();
        let Some(snapshot) = model.snapshot else {
            return;
        };
        let s = scale;
        let stock = &snapshot.stockpile;

        // Resource coins with count tabs, top-right: wood and food always, others once owned.
        let shown: Vec<(ResourceKind, f64)> = snapshot
            .catalog
            .resources
            .iter()
            .map(|&kind| (kind, stock.amount(kind)))
            .filter(|(kind, amount)| {
                matches!(kind, ResourceKind::Wood | ResourceKind::Food) || *amount >= 1.0
            })
            .collect();
        let d = 46.0 * s;
        let step = d + 18.0 * s;
        let per_row = ((width - 115.0 * s) / step).floor().max(1.0) as usize;
        for (index, (kind, amount)) in shown.iter().enumerate() {
            let row = index / per_row;
            let count = (shown.len() - row * per_row).min(per_row);
            let x = width - 16.0 * s - (count - index % per_row) as f32 * step + (step - d) / 2.0;
            let y = (10.0 + row as f32 * 82.0) * s;
            let icon = resource_icon(*kind);
            self.coin(atlas, icon, [x, y, d, d], true, false);
            let text = format!("{}", amount.floor() as i64);
            let tab = (Self::text_width(atlas, &text, 13.0 * s) + 14.0 * s).max(26.0 * s);
            self.shape(
                [x + d / 2.0 - tab / 2.0, y + d - 2.0 * s, tab, 18.0 * s],
                GLASS,
                1.0,
                9.0 * s,
            );
            self.text(
                atlas,
                &text,
                (x + d / 2.0, y + d + 12.0 * s),
                13.0 * s,
                INK,
                true,
            );
            self.text(
                atlas,
                kind.name(),
                (x + d / 2.0, y + d + 30.0 * s),
                10.0 * s,
                INK,
                true,
            );
        }

        let header_bottom = (shown.len().div_ceil(per_row) as f32 * 82.0 + 8.0) * s;
        let shared_label = "Resources shared across islands";
        self.text(
            atlas,
            shared_label,
            (
                width - 16.0 * s - Self::text_width(atlas, shared_label, 11.0 * s),
                header_bottom + 8.0 * s,
            ),
            11.0 * s,
            INK,
            false,
        );
        let header_bottom = header_bottom + 18.0 * s;
        let toast_top = header_bottom
            + if snapshot.simulation_speed == 0.0 {
                46.0 * s
            } else {
                4.0 * s
            };

        // Globe minimap bottom-right, gold rim, speed coins on its shoulder.
        let r = 68.0 * s;
        let (gx, gy) = (width - 18.0 * s - r * 2.0, height - 18.0 * s - r * 2.0);
        let globe = [gx, gy, r * 2.0, r * 2.0];
        let span = 38.0;
        let (cx, cz) = (15.0, 10.0);
        self.quads.push(Quad {
            rect: globe,
            uv: [
                (cx - span / 2.0) / 30.0,
                (cz - span / 2.0) / 20.0,
                (cx + span / 2.0) / 30.0,
                (cz + span / 2.0) / 20.0,
            ],
            color: [1.0; 4],
            params: [3.0, 0.0, 0.0, 0.0],
        });
        self.shape(globe, [0.79, 0.59, 0.25, 1.0], 2.0, 6.0 * s);
        let camera = Vec2::new(
            gx + r + (model.camera.x - cx) / span * r * 2.0,
            gy + r + (model.camera.y - cz) / span * r * 2.0,
        );
        self.shape(
            [camera.x - 5.0 * s, camera.y - 5.0 * s, 10.0 * s, 10.0 * s],
            [1.0, 1.0, 1.0, 0.95],
            2.0,
            2.0 * s,
        );
        self.regions.push(Region {
            rect: globe,
            action: Action::LookAt(Vec2::new(cx - span / 2.0, cz - span / 2.0)),
            enabled: true,
        });
        let speeds = [(0.0, "II"), (1.0, "1×"), (2.0, "2×")];
        for (index, (speed, label)) in speeds.into_iter().enumerate() {
            let angle = std::f32::consts::PI * (1.0 + 0.16 + index as f32 * 0.17);
            let c = 30.0 * s;
            let center = Vec2::new(
                gx + r + angle.cos() * (r + 22.0 * s),
                gy + r + angle.sin() * (r + 22.0 * s),
            );
            let rect = [center.x - c / 2.0, center.y - c / 2.0, c, c];
            let active = (snapshot.simulation_speed - speed).abs() < 1e-6;
            self.sprite(
                atlas,
                if self.hovered(rect) {
                    "coin_hover"
                } else {
                    "coin_normal"
                },
                rect,
                [1.0; 4],
            );
            if active {
                self.shape(
                    [
                        rect[0] + 4.0 * s,
                        rect[1] + 4.0 * s,
                        c - 8.0 * s,
                        c - 8.0 * s,
                    ],
                    ACCENT,
                    1.0,
                    (c - 8.0 * s) / 2.0,
                );
            }
            self.centered_label(
                atlas,
                label,
                center,
                12.0 * s,
                if active { [1.0; 4] } else { INK },
            );
            self.regions.push(Region {
                rect,
                action: Action::Speed(speed),
                enabled: true,
            });
        }

        // Reset opens native seed input with a progress-loss warning.
        {
            let text = "Reset game";
            let w = Self::text_width(atlas, text, 12.0 * s) + 28.0 * s;
            let pill = [12.0 * s, 14.0 * s, w, 28.0 * s];
            self.shape(pill, GLASS, 1.0, 14.0 * s);
            let ink = MUTED;
            self.text(
                atlas,
                text,
                (pill[0] + w / 2.0, pill[1] + 18.5 * s),
                12.0 * s,
                ink,
                true,
            );
            self.regions.push(Region {
                rect: pill,
                action: Action::Reset,
                enabled: true,
            });
        }

        let grid = [12.0 * s, 50.0 * s, 76.0 * s, 28.0 * s];
        self.shape(grid, GLASS, 1.0, 14.0 * s);
        self.text(
            atlas,
            if model.show_grid {
                "Grid: on"
            } else {
                "Grid: off"
            },
            (grid[0] + grid[2] / 2.0, grid[1] + 18.5 * s),
            12.0 * s,
            if model.show_grid { ACCENT } else { MUTED },
            true,
        );
        self.regions.push(Region {
            rect: grid,
            action: Action::Grid,
            enabled: true,
        });

        // While paused, a pill at the top says so; tapping it resumes, since a
        // stray tap on the pause coin otherwise looks like stuck villagers.
        if snapshot.simulation_speed == 0.0 {
            let text = "Paused · tap to resume";
            let w = Self::text_width(atlas, text, 14.0 * s) + 36.0 * s;
            let pill = [(width - w) / 2.0, header_bottom, w, 34.0 * s];
            let hot = self.hovered(pill);
            self.shape(
                pill,
                if hot { [1.0, 1.0, 1.0, 0.95] } else { GLASS },
                1.0,
                17.0 * s,
            );
            self.text(
                atlas,
                text,
                (width / 2.0, pill[1] + 22.0 * s),
                14.0 * s,
                ACCENT,
                true,
            );
            self.regions.push(Region {
                rect: pill,
                action: Action::Speed(1.0),
                enabled: true,
            });
        }

        // Selection: info pill plus a glass bar of command coins.
        let Some((portrait, title, detail, progress, commands)) = selection_model(snapshot, model)
        else {
            self.toast(atlas, model.toast, width, s, toast_top);
            return;
        };
        let gap = 10.0 * s;
        let margin = 12.0 * s;
        // Phones: the selection sits bottom-left beside the globe, its coins
        // wrapping into rows; wide screens keep one centred row.
        let narrow = width < 600.0 * s;
        let count = commands.len().max(1);
        let (m, per_row) = if narrow {
            let m = 48.0 * s;
            let room = gx - 10.0 * s - margin - 28.0 * s + gap;
            (m, ((room / (m + gap)).floor() as usize).max(1))
        } else {
            let m = ((width - 24.0 * s - 28.0 * s + gap) / count as f32 - gap)
                .clamp(40.0 * s, 52.0 * s);
            (m, count)
        };
        let labels =
            model.ship.is_some() || matches!(model.build, BuildUi::Categories | BuildUi::Group(_));
        let row_step = m + gap + if labels { 26.0 * s } else { 0.0 };
        let rows = count.div_ceil(per_row);
        let columns = count.min(per_row);
        let bar_width = columns as f32 * (m + gap) - gap + 28.0 * s;
        let bar_height = rows as f32 * row_step - gap + 12.0 * s;
        let bar = if narrow {
            [
                margin,
                height - 18.0 * s - bar_height,
                bar_width,
                bar_height,
            ]
        } else {
            let left = (width - bar_width) / 2.0;
            // A centred bar that would run under the globe sits above it.
            let top = if left + bar_width > gx - 8.0 * s {
                gy - 48.0 * s - bar_height
            } else {
                height - bar_height - 18.0 * s
            };
            [left, top, bar_width, bar_height]
        };
        let mut hover_text = None;
        if !commands.is_empty() {
            self.shape(bar, GLASS, 1.0, (m + 12.0 * s) / 2.0);
            for (index, command) in commands.iter().enumerate() {
                let (column, row) = (index % per_row, index / per_row);
                let rect = [
                    bar[0] + 14.0 * s + column as f32 * (m + gap),
                    bar[1] + 6.0 * s + row as f32 * row_step,
                    m,
                    m,
                ];
                if labels {
                    let (first, second) = command
                        .label
                        .split_once(' ')
                        .unwrap_or((&command.label, ""));
                    for (line, text) in [first, second].into_iter().enumerate() {
                        self.text(
                            atlas,
                            text,
                            (
                                rect[0] + m / 2.0,
                                rect[1] + m + (11.0 + line as f32 * 11.0) * s,
                            ),
                            10.0 * s,
                            INK,
                            true,
                        );
                    }
                }
                // The hit area spans half the gap on each side, so sweeping
                // across the bar never falls back to the selection text.
                let hit = [rect[0] - gap / 2.0, rect[1] - gap / 2.0, m + gap, m + gap];
                let hot = self.hovered(hit);
                if hot {
                    hover_text = Some((command.label.clone(), command.detail.clone()));
                }
                self.coin(
                    atlas,
                    command.icon,
                    rect,
                    command.enabled,
                    hot && command.enabled,
                );
                // An unavailable coin still answers a tap, with the reason:
                // phones have no hover to show it.
                self.regions.push(Region {
                    rect: hit,
                    action: if command.enabled {
                        command.action.clone()
                    } else {
                        Action::Explain(format!("{}: {}", command.label, command.detail))
                    },
                    enabled: true,
                });
            }
        }
        // A separate, ordered row of waiting tasks. Tapping any coin cancels
        // that task and refunds its paid inputs; the active task stays above.
        let queued = selection::queued_commands(snapshot, model);
        let mut queue_height = if queued.is_empty() { 0.0 } else { 72.0 * s };
        if !queued.is_empty() {
            let queue_width = (queued.len() as f32 * 44.0 + 24.0).max(168.0) * s;
            let left = if narrow {
                margin
            } else {
                (width - queue_width) / 2.0
            };
            let mut top = bar[1] - queue_height;
            if narrow && left + queue_width > gx - 8.0 * s {
                top = top.min(gy - 128.0 * s);
                queue_height = bar[1] - top;
            }
            self.shape([left, top, queue_width, 64.0 * s], GLASS, 1.0, 20.0 * s);
            self.text(
                atlas,
                "Queued · tap to cancel",
                (left + 12.0 * s, top + 15.0 * s),
                10.0 * s,
                MUTED,
                false,
            );
            for (index, command) in queued.iter().enumerate() {
                let rect = [
                    left + (12.0 + index as f32 * 44.0) * s,
                    top + 22.0 * s,
                    36.0 * s,
                    36.0 * s,
                ];
                let hit = [rect[0] - 4.0 * s, rect[1] - 4.0 * s, 44.0 * s, 44.0 * s];
                let hot = self.hovered(hit);
                self.coin(atlas, command.icon, rect, true, hot);
                if hot {
                    hover_text = Some((command.label.clone(), command.detail.clone()));
                }
                self.regions.push(Region {
                    rect: hit,
                    action: command.action.clone(),
                    enabled: true,
                });
            }
        }
        // One width for every text this selection can show, so hovering
        // commands changes the words but never resizes the pill.
        let widest = std::iter::once((&title, &detail))
            .chain(
                commands
                    .iter()
                    .chain(queued.iter())
                    .map(|c| (&c.label, &c.detail)),
            )
            .map(|(t, d)| {
                Self::text_width(atlas, t, 15.0 * s).max(Self::text_width(atlas, d, 12.0 * s))
            })
            .fold(0.0, f32::max);
        let info_width = (widest + 84.0 * s).max(200.0 * s).min(width - 2.0 * margin);
        let (title, detail) = hover_text.unwrap_or((title, detail));
        let detail_lines = Self::wrapped_lines(atlas, &detail, 12.0 * s, info_width - 80.0 * s);
        let extra = (detail_lines.len().saturating_sub(1)) as f32 * 14.0 * s;
        let info = if narrow {
            // Above the coins; lifted clear of the speed coins when it is
            // wide enough to reach over the globe.
            let mut top = bar[1] - queue_height - 64.0 * s - extra;
            if margin + info_width > gx - 10.0 * s {
                top = top.min(gy - 56.0 * s - 60.0 * s - extra);
            }
            [margin, top, info_width, 52.0 * s + extra]
        } else {
            [
                (width - info_width) / 2.0,
                bar[1] - queue_height - 64.0 * s - extra,
                info_width,
                52.0 * s + extra,
            ]
        };
        self.shape(info, GLASS, 1.0, 26.0 * s);
        self.sprite(
            atlas,
            portrait,
            [info[0] + 10.0 * s, info[1] + 6.0 * s, 40.0 * s, 40.0 * s],
            [1.0; 4],
        );
        self.text(
            atlas,
            &title,
            (info[0] + 64.0 * s, info[1] + 23.0 * s),
            15.0 * s,
            INK,
            false,
        );
        for (line, text) in detail_lines.iter().enumerate() {
            self.text(
                atlas,
                text,
                (
                    info[0] + 64.0 * s,
                    info[1] + (40.0 + line as f32 * 14.0) * s,
                ),
                12.0 * s,
                MUTED,
                false,
            );
        }
        if let Some(progress) = progress {
            let track = [
                info[0] + 64.0 * s,
                info[1] + info[3] - 7.0 * s,
                info_width - 84.0 * s,
                3.0 * s,
            ];
            self.shape(track, [0.24, 0.2, 0.16, 0.15], 1.0, 1.5 * s);
            self.shape(
                [track[0], track[1], track[2] * progress.min(1.0), track[3]],
                ACCENT,
                1.0,
                1.5 * s,
            );
        }
        self.toast(atlas, model.toast, width, s, toast_top);
    }
}
