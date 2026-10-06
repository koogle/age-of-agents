//! Resource totals, selected commands and their responsive placement.
use super::*;

impl Hud {
    /// Lays out the whole interface for this frame.
    pub fn layout(&mut self, atlas: &Atlas, model: &Model, width: f32, height: f32, scale: f32) {
        self.quads.clear();
        self.regions.clear();
        self.cargo_strip = None;
        let Some(snapshot) = model.snapshot else {
            return;
        };
        let s = scale;
        let narrow = width < 600.0 * s || height < 500.0 * s;
        let stock =
            &snapshot.inventories[model.resource_island.min(snapshot.inventories.len() - 1)];

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
        let mut resource_hint = None;
        let row_height = 66.0;
        let d = 46.0 * s;
        let step = d + 18.0 * s;
        let per_row = ((width - 115.0 * s) / step).floor().max(1.0) as usize;
        for (index, (kind, amount)) in shown.iter().enumerate() {
            let row = index / per_row;
            let count = (shown.len() - row * per_row).min(per_row);
            let x = width - 16.0 * s - (count - index % per_row) as f32 * step + (step - d) / 2.0;
            let y = (10.0 + row as f32 * row_height) * s;
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
            let hit = [x, y, d, d + 16.0 * s];
            let mut name = kind.name().to_owned();
            name[..1].make_ascii_uppercase();
            if self.hovered(hit) {
                resource_hint = Some(name.clone());
            }
            self.regions.push(Region {
                rect: hit,
                action: Action::Explain(name),
                enabled: true,
            });
        }

        let header_bottom = (shown.len().div_ceil(per_row) as f32 * row_height + 8.0) * s;
        let label = format!(
            "Island {} · shore + nearby ships",
            model.resource_island + 1
        );
        let shared_label = label.as_str();
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
        let r = if narrow { 40.0 } else { 68.0 } * s;
        let edge = if narrow { 12.0 } else { 18.0 } * s;
        let (gx, gy) = (width - edge - r * 2.0, height - edge - r * 2.0);
        // Reserve the full time-control row above the globe on compact screens.
        let navigation_left = if narrow { gx - 28.0 * s } else { gx };
        let globe = [gx, gy, r * 2.0, r * 2.0];
        self.map_size =
            Vec2::new(snapshot.columns as f32, snapshot.rows as f32) * crate::terrain::CELL;
        let map = minimap::Minimap::new(self.map_size);
        self.quads.push(map.quad(globe));
        self.shape(globe, [0.79, 0.59, 0.25, 1.0], 2.0, 6.0 * s);
        let camera = Vec2::new(gx, gy) + map.local_of(model.camera) * r * 2.0;
        self.shape(
            [camera.x - 5.0 * s, camera.y - 5.0 * s, 10.0 * s, 10.0 * s],
            [1.0, 1.0, 1.0, 0.95],
            2.0,
            2.0 * s,
        );
        self.regions.push(Region {
            rect: globe,
            action: Action::LookAt(map.world_at(Vec2::splat(0.5))),
            enabled: true,
        });
        let speeds = [(0.0, "II"), (1.0, "1×"), (2.0, "2×")];
        for (index, (speed, label)) in speeds.into_iter().enumerate() {
            let angle = std::f32::consts::PI * (1.0 + 0.16 + index as f32 * 0.17);
            let c = 30.0 * s;
            let center = if narrow {
                Vec2::new(gx + (-10.0 + index as f32 * 36.0) * s, gy - 26.0 * s)
            } else {
                Vec2::new(
                    gx + r + angle.cos() * (r + 22.0 * s),
                    gy + r + angle.sin() * (r + 22.0 * s),
                )
            };
            let rect = [center.x - c / 2.0, center.y - c / 2.0, c, c];
            let hit = if narrow {
                [center.x - 18.0 * s, center.y - 22.0 * s, 36.0 * s, 44.0 * s]
            } else {
                rect
            };
            let active = (snapshot.simulation_speed - speed).abs() < 1e-6;
            self.sprite(
                atlas,
                if self.hovered(hit) {
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
                rect: hit,
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
            self.toast(
                atlas,
                model.toast.or(resource_hint.as_deref()),
                width,
                s,
                toast_top,
            );
            return;
        };
        let gap = if narrow { 8.0 } else { 10.0 } * s;
        let margin = 12.0 * s;
        let padding = if narrow { 8.0 } else { 14.0 } * s;
        let selection_width = navigation_left - margin - 12.0 * s;
        // Actions and navigation share the bottom band. Longer menus grow up
        // within the left column, then collapse when placement is selected.
        let count = commands.len().max(1);
        let (m, per_row) = if narrow {
            let m = 44.0 * s;
            let room = selection_width - 2.0 * padding + gap;
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
        let bar_width = columns as f32 * (m + gap) - gap + 2.0 * padding;
        let bar_height = rows as f32 * row_step - gap + 12.0 * s;
        let bar = if narrow {
            [margin, height - edge - bar_height, bar_width, bar_height]
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
                    bar[0] + padding + column as f32 * (m + gap),
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
                let hit = [rect[0] - gap / 2.0, rect[1] - gap / 2.0, m + gap, row_step];
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
                if matches!(command.action, Action::Research(tech) if snapshot.researched_technologies.contains(&tech))
                {
                    self.sprite(
                        atlas,
                        "coin_researched",
                        [
                            rect[0] + m - 30.0 * s,
                            rect[1] + m - 30.0 * s,
                            30.0 * s,
                            30.0 * s,
                        ],
                        [1.0; 4],
                    );
                }
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
        if matches!(model.build, BuildUi::Group(_)) {
            let hint = hover_text.map(|(title, detail)| format!("{title}: {detail}"));
            self.toast(
                atlas,
                model.toast.or(resource_hint.as_deref()).or(hint.as_deref()),
                width,
                s,
                toast_top,
            );
            return;
        }
        // A separate, ordered row of waiting tasks. Tapping any coin cancels
        // that task and refunds its paid inputs; the active task stays above.
        let queued = selection::queued_commands(snapshot, model);
        let queue_columns = if narrow {
            ((selection_width - 16.0 * s) / (44.0 * s)).floor().max(1.0) as usize
        } else {
            queued.len().max(1)
        };
        let queue_height = if queued.is_empty() {
            0.0
        } else {
            (28.0 + queued.len().div_ceil(queue_columns) as f32 * 44.0) * s
        };
        if !queued.is_empty() {
            let queue_width = if narrow {
                selection_width.min(
                    ((queued.len().min(queue_columns) as f32 * 44.0 + 16.0) * s).max(168.0 * s),
                )
            } else {
                (queued.len() as f32 * 44.0 + 24.0).max(168.0) * s
            };
            let left = if narrow {
                margin
            } else {
                (width - queue_width) / 2.0
            };
            let top = bar[1] - queue_height;
            self.shape(
                [left, top, queue_width, queue_height - 8.0 * s],
                GLASS,
                1.0,
                20.0 * s,
            );
            self.text(
                atlas,
                "Queued · tap to cancel",
                (left + if narrow { 8.0 } else { 12.0 } * s, top + 15.0 * s),
                10.0 * s,
                MUTED,
                false,
            );
            for (index, command) in queued.iter().enumerate() {
                let rect = [
                    left + (if narrow { 8.0 } else { 12.0 }
                        + (index % queue_columns) as f32 * 44.0)
                        * s,
                    top + (22.0 + (index / queue_columns) as f32 * 44.0) * s,
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
        let info_width = if narrow {
            selection_width.min((widest + 64.0 * s).max(156.0 * s))
        } else {
            (widest + 84.0 * s).max(200.0 * s).min(width - 2.0 * margin)
        };
        let (title, detail) = hover_text.unwrap_or((title, detail));
        let text_offset = if narrow { 52.0 } else { 64.0 } * s;
        let text_room = info_width - text_offset - if narrow { 12.0 } else { 16.0 } * s;
        let title_size = if narrow { 14.0 } else { 15.0 } * s;
        let title_lines = if narrow {
            Self::wrapped_lines(atlas, &title, title_size, text_room)
        } else {
            vec![title]
        };
        let detail_lines = Self::wrapped_lines(atlas, &detail, 12.0 * s, text_room);
        let title_extra = title_lines.len().saturating_sub(1) as f32 * 16.0 * s;
        let extra = title_extra + detail_lines.len().saturating_sub(1) as f32 * 14.0 * s;
        let info_height = if narrow { 48.0 } else { 52.0 } * s + extra;
        let info_gap = if narrow { 8.0 } else { 12.0 } * s;
        let info = [
            if narrow {
                margin
            } else {
                (width - info_width) / 2.0
            },
            bar[1] - queue_height - info_gap - info_height,
            info_width,
            info_height,
        ];
        self.cargo_panel(atlas, model, width, info[1] - 8.0 * s, s, narrow);
        self.shape(info, GLASS, 1.0, 26.0 * s);
        let portrait_size = if narrow { 32.0 } else { 40.0 } * s;
        self.sprite(
            atlas,
            portrait,
            [
                info[0] + 10.0 * s,
                info[1] + if narrow { 8.0 } else { 6.0 } * s,
                portrait_size,
                portrait_size,
            ],
            [1.0; 4],
        );
        for (line, text) in title_lines.iter().enumerate() {
            self.text(
                atlas,
                text,
                (
                    info[0] + text_offset,
                    info[1] + (if narrow { 20.0 } else { 23.0 } + line as f32 * 16.0) * s,
                ),
                title_size,
                INK,
                false,
            );
        }
        for (line, text) in detail_lines.iter().enumerate() {
            self.text(
                atlas,
                text,
                (
                    info[0] + text_offset,
                    info[1]
                        + title_extra
                        + (if narrow { 36.0 } else { 40.0 } + line as f32 * 14.0) * s,
                ),
                12.0 * s,
                MUTED,
                false,
            );
        }
        if let Some(progress) = progress {
            let track = [
                info[0] + text_offset,
                info[1] + info[3] - 7.0 * s,
                text_room,
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
        self.toast(
            atlas,
            model.toast.or(resource_hint.as_deref()),
            width,
            s,
            toast_top,
        );
    }
}
