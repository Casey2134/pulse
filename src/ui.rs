use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Gauge},
    Frame,
};

use crate::app::{App, InputMode, Panel};
use crate::models::{format_bytes, format_uptime, ContainerStatus, NodeStatus};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(8),     // Main panels
            Constraint::Length(7),  // Detail panel
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    draw_nodes(frame, app, main_chunks[0]);
    draw_containers(frame, app, main_chunks[1]);
    draw_detail_panel(frame, app, chunks[2]);
    draw_status_bar(frame, app, chunks[3]);

    if app.show_help {
        draw_help_popup(frame);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let (nodes_online, nodes_total) = app.nodes_summary();
    let (containers_running, containers_total) = app.containers_summary();

    let title = vec![
        Span::styled(" PULSE ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| "),
        Span::styled(
            format!("Nodes: {}/{}", nodes_online, nodes_total),
            Style::default().fg(if nodes_online == nodes_total {
                Color::Green
            } else {
                Color::Yellow
            }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Containers: {}/{}", containers_running, containers_total),
            Style::default().fg(if containers_running == containers_total {
                Color::Green
            } else {
                Color::Yellow
            }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Sort: {} {}", app.sort_field.label(), if app.sort_ascending { "^" } else { "v" }),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Refresh: {}", app.time_since_refresh()),
            Style::default().fg(Color::Gray),
        ),
    ];

    let header = Paragraph::new(Line::from(title)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(header, area);
}

fn draw_nodes(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Nodes;
    let nodes = app.filtered_nodes();

    let items: Vec<ListItem> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let (status_icon, status_color) = match node.status {
                NodeStatus::Online => ("●", Color::Green),
                NodeStatus::Offline => ("○", Color::Red),
            };

            let cpu_bar = create_mini_bar(node.cpu_usage, 8);
            let mem_bar = create_mini_bar(node.memory_percent(), 8);

            let selected = i == app.node_index && is_active;
            let prefix = if selected { ">" } else { " " };

            let content = Line::from(vec![
                Span::raw(prefix),
                Span::styled(status_icon, Style::default().fg(status_color)),
                Span::raw(format!(" {:<10} ", truncate(&node.name, 10))),
                Span::styled("CPU", Style::default().fg(Color::Gray)),
                Span::raw(cpu_bar),
                Span::raw(" "),
                Span::styled("MEM", Style::default().fg(Color::Gray)),
                Span::raw(mem_bar),
            ]);

            if selected {
                ListItem::new(content).style(Style::default().bg(Color::DarkGray))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = format!(
        " Nodes ({}/{}) ",
        app.nodes_summary().0,
        app.nodes_summary().1
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn draw_containers(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Containers;
    let containers = app.filtered_containers();

    let items: Vec<ListItem> = containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let (status_icon, status_color) = match container.status {
                ContainerStatus::Running => ("●", Color::Green),
                ContainerStatus::Stopped => ("○", Color::Red),
            };

            let type_color = match container.container_type {
                crate::models::ContainerType::VM => Color::Magenta,
                crate::models::ContainerType::LXC => Color::Blue,
            };

            let selected = i == app.container_index && is_active;
            let prefix = if selected { ">" } else { " " };

            let content = Line::from(vec![
                Span::raw(prefix),
                Span::styled(status_icon, Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(
                    format!("{:<3}", container.type_label()),
                    Style::default().fg(type_color),
                ),
                Span::raw(format!(" {:<12} ", truncate(&container.name, 12))),
                Span::styled(
                    format!("{:<8}", truncate(&container.node, 8)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(format!(" {:>5.1}% ", container.cpu_usage)),
                Span::raw(format!("{:>8}", format_bytes(container.memory_used))),
            ]);

            if selected {
                ListItem::new(content).style(Style::default().bg(Color::DarkGray))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = format!(
        " Containers ({}/{}) ",
        app.containers_summary().0,
        app.containers_summary().1
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn draw_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.active_panel {
        Panel::Nodes => {
            if let Some(node) = app.selected_node() {
                draw_node_details(frame, node, inner);
            } else {
                let msg = Paragraph::new("No node selected")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(msg, inner);
            }
        }
        Panel::Containers => {
            if let Some(container) = app.selected_container() {
                draw_container_details(frame, container, inner);
            } else {
                let msg = Paragraph::new("No container selected")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(msg, inner);
            }
        }
    }
}

fn draw_node_details(frame: &mut Frame, node: &crate::models::Node, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);

    // Title line
    let status_text = match node.status {
        NodeStatus::Online => Span::styled("Online", Style::default().fg(Color::Green)),
        NodeStatus::Offline => Span::styled("Offline", Style::default().fg(Color::Red)),
    };

    let title_line = Line::from(vec![
        Span::styled(&node.name, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | Status: "),
        status_text,
        Span::raw(" | Uptime: "),
        Span::raw(format_uptime(node.uptime)),
    ]);
    frame.render_widget(Paragraph::new(title_line), chunks[0]);

    // CPU gauge
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU"))
        .gauge_style(Style::default().fg(cpu_color(node.cpu_usage)))
        .percent(node.cpu_usage.min(100.0) as u16)
        .label(format!("{:.1}%", node.cpu_usage));
    frame.render_widget(cpu_gauge, chunks[1]);

    // Memory gauge
    let mem_pct = node.memory_percent();
    let mem_label = format!(
        "{:.1}% ({} / {})",
        mem_pct,
        format_bytes(node.memory_used),
        format_bytes(node.memory_total)
    );
    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory"))
        .gauge_style(Style::default().fg(cpu_color(mem_pct)))
        .percent(mem_pct.min(100.0) as u16)
        .label(mem_label);
    frame.render_widget(mem_gauge, chunks[2]);
}

fn draw_container_details(frame: &mut Frame, container: &crate::models::Container, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);

    // Title line
    let type_span = match container.container_type {
        crate::models::ContainerType::VM => {
            Span::styled("QEMU VM", Style::default().fg(Color::Magenta))
        }
        crate::models::ContainerType::LXC => {
            Span::styled("LXC Container", Style::default().fg(Color::Blue))
        }
    };

    let status_span = match container.status {
        ContainerStatus::Running => Span::styled("Running", Style::default().fg(Color::Green)),
        ContainerStatus::Stopped => Span::styled("Stopped", Style::default().fg(Color::Red)),
    };

    let title_line = Line::from(vec![
        Span::styled(&container.name, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!(" (ID: {}) | ", container.vmid)),
        type_span,
        Span::raw(" | Node: "),
        Span::raw(&container.node),
        Span::raw(" | "),
        status_span,
        Span::raw(" | Uptime: "),
        Span::raw(format_uptime(container.uptime)),
    ]);
    frame.render_widget(Paragraph::new(title_line), chunks[0]);

    // CPU gauge
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU"))
        .gauge_style(Style::default().fg(cpu_color(container.cpu_usage)))
        .percent(container.cpu_usage.min(100.0) as u16)
        .label(format!("{:.1}%", container.cpu_usage));
    frame.render_widget(cpu_gauge, chunks[1]);

    // Memory gauge
    let mem_pct = container.memory_percent();
    let mem_label = format!(
        "{:.1}% ({} / {})",
        mem_pct,
        format_bytes(container.memory_used),
        format_bytes(container.memory_max)
    );
    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory"))
        .gauge_style(Style::default().fg(cpu_color(mem_pct)))
        .percent(mem_pct.min(100.0) as u16)
        .label(mem_label);
    frame.render_widget(mem_gauge, chunks[2]);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (left_text, style) = match app.input_mode {
        InputMode::Search => {
            let text = format!(" Search: {}_ ", app.search_query);
            (text, Style::default().fg(Color::Yellow))
        }
        InputMode::Normal => {
            if let Some(ref error) = app.error_message {
                (format!(" Error: {} ", error), Style::default().fg(Color::Red))
            } else {
                let text =
                    " q:Quit  Tab:Panel  j/k:Nav  r:Refresh  s:Sort  /:Search  ?:Help ".to_string();
                (text, Style::default().fg(Color::Gray))
            }
        }
    };

    let status = Paragraph::new(left_text).style(style);
    frame.render_widget(status, area);
}

fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(50, 60, frame.area());

    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q      ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  Tab    ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch between panels"),
        ]),
        Line::from(vec![
            Span::styled("  j/Down ", Style::default().fg(Color::Cyan)),
            Span::raw("Move selection down"),
        ]),
        Line::from(vec![
            Span::styled("  k/Up   ", Style::default().fg(Color::Cyan)),
            Span::raw("Move selection up"),
        ]),
        Line::from(vec![
            Span::styled("  r      ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh data"),
        ]),
        Line::from(vec![
            Span::styled("  s      ", Style::default().fg(Color::Cyan)),
            Span::raw("Cycle sort field"),
        ]),
        Line::from(vec![
            Span::styled("  S      ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle sort order"),
        ]),
        Line::from(vec![
            Span::styled("  /      ", Style::default().fg(Color::Cyan)),
            Span::raw("Enter search mode"),
        ]),
        Line::from(vec![
            Span::styled("  Esc    ", Style::default().fg(Color::Cyan)),
            Span::raw("Clear search / Exit mode"),
        ]),
        Line::from(vec![
            Span::styled("  ?      ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(help, area);
}

// Helper functions

fn create_mini_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}~", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}

fn cpu_color(percent: f64) -> Color {
    if percent >= 90.0 {
        Color::Red
    } else if percent >= 70.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
