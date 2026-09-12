use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

const TOP_LEFT: &str = "┏";
const TOP_RIGHT: &str = "┓";
const BOTTOM_LEFT: &str = "┗";
const BOTTOM_RIGHT: &str = "┛";
const VERTICAL: &str = "┃";
const HORIZONTAL: &str = "━";

const DIM_TOP_LEFT: &str = "┌";
const DIM_TOP_RIGHT: &str = "┐";
const DIM_BOTTOM_LEFT: &str = "└";
const DIM_BOTTOM_RIGHT: &str = "┘";
const DIM_VERTICAL: &str = "│";
const DIM_HORIZONTAL: &str = "─";

/// Renders an animated gradient border if focused, otherwise a dim standard border.
pub fn render_gradient_border(
    buf: &mut Buffer,
    area: Rect,
    animation_frame: u64,
    focused: bool,
    is_running: bool,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    if !focused {
        render_dim_border(buf, area);
        return;
    }

    // Base time factor for animations
    let t = animation_frame as f64 * 0.08;

    // Corners
    draw_border_cell(buf, area.x, area.y, area, t, TOP_LEFT, is_running);
    draw_border_cell(
        buf,
        area.x + area.width - 1,
        area.y,
        area,
        t,
        TOP_RIGHT,
        is_running,
    );
    draw_border_cell(
        buf,
        area.x,
        area.y + area.height - 1,
        area,
        t,
        BOTTOM_LEFT,
        is_running,
    );
    draw_border_cell(
        buf,
        area.x + area.width - 1,
        area.y + area.height - 1,
        area,
        t,
        BOTTOM_RIGHT,
        is_running,
    );

    // Top and Bottom
    for x in (area.x + 1)..(area.x + area.width - 1) {
        draw_border_cell(buf, x, area.y, area, t, HORIZONTAL, is_running);
        draw_border_cell(
            buf,
            x,
            area.y + area.height - 1,
            area,
            t,
            HORIZONTAL,
            is_running,
        );
    }

    // Left and Right
    for y in (area.y + 1)..(area.y + area.height - 1) {
        draw_border_cell(buf, area.x, y, area, t, VERTICAL, is_running);
        draw_border_cell(
            buf,
            area.x + area.width - 1,
            y,
            area,
            t,
            VERTICAL,
            is_running,
        );
    }
}

fn draw_border_cell(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    area: Rect,
    t: f64,
    symbol: &str,
    is_running: bool,
) {
    if x < buf.area.width && y < buf.area.height {
        let cell = &mut buf[(x, y)];

        // Only overwrite the symbol if it's a standard border character or empty/space
        // This preserves titles rendered by Block::title()
        let current_sym = cell.symbol();
        if current_sym == " " || current_sym.is_empty() || is_border_symbol(current_sym) {
            cell.set_symbol(symbol);
        }

        let color = get_color(
            x - area.x,
            y - area.y,
            area.width,
            area.height,
            t,
            is_running,
        );

        // Apply color and reset background/modifiers to prevent artifacts
        cell.fg = color;
        cell.bg = Color::Reset;
        cell.modifier = Modifier::empty();
    }
}

fn is_border_symbol(sym: &str) -> bool {
    matches!(
        sym,
        "┌" | "┐" | "└" | "┘" | "│" | "─" | // Light
        "┏" | "┓" | "┗" | "┛" | "┃" | "━" | // Thick
        "╔" | "╗" | "╚" | "╝" | "║" | "═" | // Double
        "╭" | "╮" | "╯" | "╰" // Rounded
    )
}

fn get_perimeter_pos(x: u16, y: u16, width: u16, height: u16) -> f64 {
    let w = width.saturating_sub(1) as f64;
    let h = height.saturating_sub(1) as f64;
    let xf = x as f64;
    let yf = y as f64;

    if yf == 0.0 {
        xf
    } else if xf == w {
        w + yf
    } else if yf == h {
        w + h + (w - xf)
    } else {
        w + h + w + (h - yf)
    }
}

fn calculate_pulse(pos: f64, perimeter: f64, t_scaled: f64, width: f64, t: f64) -> f64 {
    if perimeter <= 0.0 {
        return 0.0;
    }

    // Liquid modulation: width and position oscillate slightly
    let modulated_width = width * (1.0 + 0.2 * (t * 2.0 + pos * 0.1).sin());
    let modulated_center = (t_scaled + 2.0 * (t * 1.5 + pos * 0.05).cos()).rem_euclid(perimeter);

    let mut d = (pos - modulated_center).abs();
    if d > perimeter / 2.0 {
        d = perimeter - d;
    }

    if d < modulated_width {
        let x = d / modulated_width;
        // Quintic smoothstep for ultra-liquid feel: 10x^3 - 15x^4 + 6x^5
        let val = 1.0 - (10.0 * x.powi(3) - 15.0 * x.powi(4) + 6.0 * x.powi(5));
        val
    } else {
        0.0
    }
}

fn get_color(x: u16, y: u16, width: u16, height: u16, t: f64, is_running: bool) -> Color {
    let p_pos = get_perimeter_pos(x, y, width, height);
    let w = width.saturating_sub(1) as f64;
    let h = height.saturating_sub(1) as f64;
    let perimeter = 2.0 * (w + h);

    if perimeter <= 0.0 {
        return Color::Rgb(100, 100, 100);
    }

    let norm_p = p_pos / perimeter;
    let mut r: f64;
    let mut g: f64;
    let mut b: f64;

    if is_running {
        // RUNNING STATE: Intense, complex, highly energetic. Rapid neon pulses, sharp chromatic shifts.

        let r_base = (norm_p * 40.0 + t * 15.0).sin() * 0.5 + 0.5;
        let g_base = (norm_p * 50.0 - t * 18.0).sin() * 0.5 + 0.5;
        let b_base = (norm_p * 30.0 + t * 20.0).sin() * 0.5 + 0.5;

        // Traveling Pulses with liquid modulation
        let p1 = calculate_pulse(p_pos, perimeter, t * 40.0, 6.0, t);
        let p2 = calculate_pulse(p_pos, perimeter, -t * 30.0, 10.0, t);
        let p3 = calculate_pulse(p_pos, perimeter, t * 60.0, 3.0, t);
        let p4 = calculate_pulse(p_pos, perimeter, -t * 45.0, 8.0, t);
        let p5 = calculate_pulse(p_pos, perimeter, t * 25.0, 15.0, t);

        let is_corner = (x == 0 || x == width - 1) && (y == 0 || y == height - 1);
        let flare = if is_corner {
            ((t * 20.0).sin() * 0.5 + 0.5).powi(3) * 1.5
        } else {
            0.0
        };

        r = r_base * 0.05 + p1 * 0.1 + p2 * 1.0 + p3 * 1.0 + p4 * 0.6 + p5 * 0.2 + flare;
        g = g_base * 0.05 + p1 * 0.9 + p2 * 0.1 + p3 * 1.0 + p4 * 1.0 + p5 * 0.3 + flare;
        b = b_base * 0.05 + p1 * 1.0 + p2 * 0.9 + p3 * 1.0 + p4 * 0.1 + p5 * 0.5 + flare;

        // Overdrive for neon intensity
        r *= 1.2;
        g *= 1.2;
        b *= 1.2;

        // Lightning shimmer (Phase 4.2 extension)
        if (t * 12.0 + norm_p * 20.0).cos() > 0.97 {
            r += 0.8;
            g += 0.8;
            b += 1.0;
        }
    } else {
        // IDLE/FOCUSED STATE: Majestic, calm, ethereal glow. Overlapping sine waves.

        let phase1 = norm_p * 6.28;
        let phase2 = norm_p * 12.56;
        let phase3 = norm_p * 25.12;

        // Multi-layered waves for complexity
        let w1 = (phase1 + t * 0.2).sin() * 0.5 + 0.5;
        let w2 = (phase2 - t * 0.15).cos() * 0.5 + 0.5;
        let w3 = (phase3 + t * 0.4).sin() * 0.5 + 0.5;
        let w4 = (phase1 * 0.5 - t * 0.1).cos() * 0.5 + 0.5;

        // Base color palette: Deep violets and cosmic blues
        let r_base = 0.05 + w1 * 0.2 + w4 * 0.1;
        let g_base = 0.1 + w2 * 0.2 + w3 * 0.1;
        let b_base = 0.3 + w1 * 0.3 + w4 * 0.3;

        // Subtle traveling glow "sweep"
        let sweep_pos = (t * 0.3).rem_euclid(1.0);
        let mut sweep_dist = (norm_p - sweep_pos).abs();
        if sweep_dist > 0.5 {
            sweep_dist = 1.0 - sweep_dist;
        }
        let sweep = (1.0 - (sweep_dist / 0.15).min(1.0)).powi(4);

        // Add the sweep and a tiny bit of high-frequency shimmer
        let shimmer = (phase3 * 2.0 + t * 3.0).sin() * 0.03 + 0.03;

        r = r_base + sweep * 0.15 + shimmer;
        g = g_base + sweep * 0.3 + shimmer;
        b = b_base + sweep * 0.5 + shimmer;

        // Majestic breathing effect
        let breath = (t * 0.6).sin() * 0.12 + 0.88;
        r *= breath;
        g *= breath;
        b *= breath;
    }

    // Convert to RGB888 with clamping
    let r_u8 = (r * 255.0).clamp(0.0, 255.0) as u8;
    let g_u8 = (g * 255.0).clamp(0.0, 255.0) as u8;
    let b_u8 = (b * 255.0).clamp(0.0, 255.0) as u8;

    Color::Rgb(r_u8, g_u8, b_u8)
}

fn render_dim_border(buf: &mut Buffer, area: Rect) {
    let style = Style::default().fg(Color::Indexed(240));

    // Corners
    draw_dim_cell(buf, area.x, area.y, DIM_TOP_LEFT, style);
    draw_dim_cell(buf, area.x + area.width - 1, area.y, DIM_TOP_RIGHT, style);
    draw_dim_cell(
        buf,
        area.x,
        area.y + area.height - 1,
        DIM_BOTTOM_LEFT,
        style,
    );
    draw_dim_cell(
        buf,
        area.x + area.width - 1,
        area.y + area.height - 1,
        DIM_BOTTOM_RIGHT,
        style,
    );

    // Top and Bottom
    for x in (area.x + 1)..(area.x + area.width - 1) {
        draw_dim_cell(buf, x, area.y, DIM_HORIZONTAL, style);
        draw_dim_cell(buf, x, area.y + area.height - 1, DIM_HORIZONTAL, style);
    }

    // Left and Right
    for y in (area.y + 1)..(area.y + area.height - 1) {
        draw_dim_cell(buf, area.x, y, DIM_VERTICAL, style);
        draw_dim_cell(buf, area.x + area.width - 1, y, DIM_VERTICAL, style);
    }
}

fn draw_dim_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if x < buf.area.width && y < buf.area.height {
        let cell = &mut buf[(x, y)];

        let current_sym = cell.symbol();
        if current_sym == " " || current_sym.is_empty() || is_border_symbol(current_sym) {
            cell.set_symbol(symbol);
        }

        // Explicitly reset to ensure no artifacts
        cell.bg = Color::Reset;
        cell.modifier = Modifier::empty();

        if let Some(fg) = style.fg {
            cell.fg = fg;
        }
    }
}
