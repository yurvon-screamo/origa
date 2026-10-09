use tracing::debug;

use super::StrokeData;

pub fn parse_stroke_paths(svg: &str) -> Vec<StrokeData> {
    let mut strokes = Vec::new();
    let mut pos = 0;
    while let Some(rel_start) = svg[pos..].find("<path") {
        let abs_start = pos + rel_start;
        let rest = &svg[abs_start..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let path_tag = &rest[..=tag_end.min(rest.len() - 1)];
        if !path_tag.contains("class=\"bg\"")
            && !path_tag.contains("class='bg'")
            && let Some(d) = extract_attribute(path_tag, "d")
        {
            debug!(stroke_index = strokes.len(), d = %d, "Parsed stroke path");
            strokes.push(StrokeData { d });
        }
        pos = abs_start + tag_end + 1;
    }
    debug!(
        total_strokes = strokes.len(),
        "Finished parsing SVG strokes"
    );
    strokes
}

fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=\"", attr), format!("{}='", attr)];
    for pattern in patterns {
        if let Some(start) = tag.find(&pattern) {
            let value_start = start + pattern.len();
            let quote = &pattern[pattern.len() - 1..pattern.len()];
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    ClosePath,
}

/// A parsed cubic segment: the command, the position after its
/// coordinates, the new current point and the (absolute) second control
/// point for smooth-curve reflection.
type ParsedCurve = (PathCommand, usize, (f64, f64), (f64, f64));

pub fn parse_svg_path_commands(d: &str) -> Vec<PathCommand> {
    let mut commands = Vec::new();
    let chars: Vec<char> = d.chars().collect();
    let mut pos = 0;
    let mut current_cmd = 'M';
    let mut current_pos = (0.0_f64, 0.0_f64);
    // Second control point of the previous cubic curve — the reflection
    // source for smooth S/s commands (SVG spec).
    let mut prev_cubic_ctrl2: Option<(f64, f64)> = None;

    while pos < chars.len() {
        // Separators come FIRST: in the spaced KanjiVG format
        // ("M 17.88,20.29 c 1.91,…") the command letter sits behind a
        // space. Reading it as an (assumed implicit LineTo) coordinate
        // used to abort the parser right after the MoveTo — the canvas
        // then had nothing to draw (a blank practice canvas for every
        // kanji whose art uses the spaced format).
        pos = skip_whitespace(&chars, pos);
        if pos >= chars.len() {
            break;
        }
        let c = chars[pos];
        if c.is_ascii_alphabetic() {
            current_cmd = c;
            pos += 1;
            // No `continue` here: Z has no coordinates, so deferring the
            // execution to the next iteration would swallow it entirely.
        }
        let Some(segment) = parse_segment(current_cmd, current_pos, prev_cubic_ctrl2, &chars, pos)
        else {
            break;
        };
        current_pos = segment.current;
        if let CubicCtrl2::Set(point) = segment.cubic_ctrl2 {
            prev_cubic_ctrl2 = Some(point);
        } else {
            prev_cubic_ctrl2 = None;
        }
        if let Some(command) = segment.command {
            commands.push(command);
        }
        pos = segment.new_pos;
        current_cmd = segment.next_cmd;
    }
    commands
}

/// How a parsed segment changes the smooth-curve reflection state.
#[derive(Clone, Copy)]
enum CubicCtrl2 {
    /// Not a cubic command (or an unknown one) — the previous control
    /// point is forgotten: the spec reflects only through an unbroken
    /// C/S chain.
    Reset,
    /// A C/c/S/s segment — the new second control point (absolute).
    Set((f64, f64)),
}

struct ParsedSegment {
    /// `None` for skipped unknown-command characters.
    command: Option<PathCommand>,
    new_pos: usize,
    current: (f64, f64),
    /// The implicit command for coordinate groups that follow without a
    /// letter (after M it is L per the SVG spec).
    next_cmd: char,
    cubic_ctrl2: CubicCtrl2,
}

/// Parses the coordinate group of `cmd` starting at `pos` (which points
/// at the first character after the command letter). All command-letter
/// branching lives here so the driver loop above stays trivial.
fn parse_segment(
    cmd: char,
    current_pos: (f64, f64),
    prev_cubic_ctrl2: Option<(f64, f64)>,
    chars: &[char],
    pos: usize,
) -> Option<ParsedSegment> {
    match cmd {
        'M' | 'm' => {
            let (command, new_pos, current, next_cmd) = parse_move(cmd, current_pos, chars, pos)?;
            Some(ParsedSegment {
                command: Some(command),
                new_pos,
                current,
                next_cmd,
                cubic_ctrl2: CubicCtrl2::Reset,
            })
        },
        'L' | 'l' => {
            let (command, new_pos, current) = parse_line(cmd, current_pos, chars, pos)?;
            Some(ParsedSegment {
                command: Some(command),
                new_pos,
                current,
                next_cmd: cmd,
                cubic_ctrl2: CubicCtrl2::Reset,
            })
        },
        'C' | 'c' => {
            let (command, new_pos, current, ctrl2) = parse_curve(cmd, current_pos, chars, pos)?;
            Some(ParsedSegment {
                command: Some(command),
                new_pos,
                current,
                next_cmd: cmd,
                cubic_ctrl2: CubicCtrl2::Set(ctrl2),
            })
        },
        'S' | 's' => {
            let (command, new_pos, current, ctrl2) =
                parse_smooth_curve(cmd, current_pos, prev_cubic_ctrl2, chars, pos)?;
            Some(ParsedSegment {
                command: Some(command),
                new_pos,
                current,
                next_cmd: cmd,
                cubic_ctrl2: CubicCtrl2::Set(ctrl2),
            })
        },
        'Z' | 'z' => Some(ParsedSegment {
            command: Some(PathCommand::ClosePath),
            // The loop has already consumed the letter, and Z carries no
            // coordinates — nothing more to advance past.
            new_pos: pos,
            current: current_pos,
            next_cmd: cmd,
            cubic_ctrl2: CubicCtrl2::Reset,
        }),
        // Unknown command letter: skip it. It cannot carry a valid
        // reflection, so the smooth-curve state resets — matching the
        // spec, where only an unbroken C/S chain reflects.
        _ => Some(ParsedSegment {
            command: None,
            new_pos: pos + 1,
            current: current_pos,
            next_cmd: cmd,
            cubic_ctrl2: CubicCtrl2::Reset,
        }),
    }
}

/// Smooth cubic (`S x2,y2 x,y` / `s …`): the first control point is the
/// reflection of the previous cubic's second control point through the
/// current position; without a previous cubic it IS the current point.
fn parse_smooth_curve(
    cmd: char,
    current_pos: (f64, f64),
    prev_cubic_ctrl2: Option<(f64, f64)>,
    chars: &[char],
    pos: usize,
) -> Option<ParsedCurve> {
    let (x2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (x, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y, pos) = parse_number(chars, skip_whitespace(chars, pos))?;

    let (abs_x2, abs_y2, abs_x, abs_y) = if cmd == 's' {
        (
            current_pos.0 + x2,
            current_pos.1 + y2,
            current_pos.0 + x,
            current_pos.1 + y,
        )
    } else {
        (x2, y2, x, y)
    };
    let (ref_x, ref_y) = match prev_cubic_ctrl2 {
        Some((cx, cy)) => (2.0 * current_pos.0 - cx, 2.0 * current_pos.1 - cy),
        None => current_pos,
    };
    Some((
        PathCommand::CurveTo(ref_x, ref_y, abs_x2, abs_y2, abs_x, abs_y),
        pos,
        (abs_x, abs_y),
        (abs_x2, abs_y2),
    ))
}

fn parse_move(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<(PathCommand, usize, (f64, f64), char)> {
    let (x, y, new_pos) = parse_coords(chars, pos)?;
    let (abs_x, abs_y) = if cmd == 'm' {
        (current_pos.0 + x, current_pos.1 + y)
    } else {
        (x, y)
    };
    let next_cmd = if cmd == 'm' { 'l' } else { 'L' };
    Some((
        PathCommand::MoveTo(abs_x, abs_y),
        new_pos,
        (abs_x, abs_y),
        next_cmd,
    ))
}

fn parse_line(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<(PathCommand, usize, (f64, f64))> {
    let (x, y, new_pos) = parse_coords(chars, pos)?;
    let (abs_x, abs_y) = if cmd == 'l' {
        (current_pos.0 + x, current_pos.1 + y)
    } else {
        (x, y)
    };
    Some((PathCommand::LineTo(abs_x, abs_y), new_pos, (abs_x, abs_y)))
}

fn parse_curve(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<ParsedCurve> {
    let (x1, y1, x2, y2, x, y, new_pos) = parse_curve_coords(chars, pos)?;
    let (abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y) = if cmd == 'c' {
        (
            current_pos.0 + x1,
            current_pos.1 + y1,
            current_pos.0 + x2,
            current_pos.1 + y2,
            current_pos.0 + x,
            current_pos.1 + y,
        )
    } else {
        (x1, y1, x2, y2, x, y)
    };
    Some((
        PathCommand::CurveTo(abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y),
        new_pos,
        (abs_x, abs_y),
        (abs_x2, abs_y2),
    ))
}

pub(crate) fn parse_coords(chars: &[char], start: usize) -> Option<(f64, f64, usize)> {
    let mut pos = skip_whitespace(chars, start);
    let (x, new_pos) = parse_number(chars, pos)?;
    pos = skip_whitespace(chars, new_pos);
    let (y, new_pos) = parse_number(chars, pos)?;
    Some((x, y, new_pos))
}

pub(crate) fn parse_curve_coords(
    chars: &[char],
    start: usize,
) -> Option<(f64, f64, f64, f64, f64, f64, usize)> {
    let (x1, pos) = parse_number(chars, skip_whitespace(chars, start))?;
    let (y1, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (x2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (x, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    Some((x1, y1, x2, y2, x, y, pos))
}

pub(crate) fn skip_whitespace(chars: &[char], start: usize) -> usize {
    let mut pos = start;
    while pos < chars.len() && (chars[pos].is_whitespace() || chars[pos] == ',') {
        pos += 1;
    }
    pos
}

pub(crate) fn parse_number(chars: &[char], start: usize) -> Option<(f64, usize)> {
    let mut pos = start;
    let mut num_str = String::new();
    if pos < chars.len() && (chars[pos] == '-' || chars[pos] == '+') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    let num: f64 = num_str.parse().ok()?;
    Some((num, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stroke_paths_extracts_d_attributes() {
        let svg = r#"<svg><path d="M 10 10 L 20 20"/><path d="M 1 2 C 3 4 5 6 7 8"/></svg>"#;
        let strokes = parse_stroke_paths(svg);
        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].d, "M 10 10 L 20 20");
        assert_eq!(strokes[1].d, "M 1 2 C 3 4 5 6 7 8");
    }

    #[test]
    fn parse_stroke_paths_skips_background_strokes() {
        let svg = r#"<path class="bg" d="M 0 0 L 1 1"/><path d="M 5 5 L 6 6"/>"#;
        let strokes = parse_stroke_paths(svg);
        assert_eq!(strokes.len(), 1, "bg stroke must be filtered out");
        assert_eq!(strokes[0].d, "M 5 5 L 6 6");
    }

    #[test]
    fn parse_stroke_paths_empty_input_yields_nothing() {
        assert!(parse_stroke_paths("").is_empty());
        assert!(parse_stroke_paths("<circle cx=\"1\"/>").is_empty());
    }

    #[test]
    fn parse_path_commands_absolute_move_line_close() {
        // Compact comma-separated format used by CDN kanji SVG files.
        let cmds = parse_svg_path_commands("M10,20L30,40Z");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 20.0),
                PathCommand::LineTo(30.0, 40.0),
                PathCommand::ClosePath,
            ]
        );
    }

    #[test]
    fn parse_path_commands_relative_coordinates() {
        // m 10 10 l 5 0 → MoveTo(10,10) then relative LineTo(15,10)
        let cmds = parse_svg_path_commands("m10,10l5,0");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 10.0),
                PathCommand::LineTo(15.0, 10.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_implicit_line_after_move() {
        // SVG allows implicit repetition: "M1,2 3,4" = M 1 2 L 3 4
        let cmds = parse_svg_path_commands("M1,2 3,4");
        assert_eq!(
            cmds,
            vec![PathCommand::MoveTo(1.0, 2.0), PathCommand::LineTo(3.0, 4.0),]
        );
    }

    #[test]
    fn parse_path_commands_absolute_cubic_bezier() {
        let cmds = parse_svg_path_commands("M0,0C1,2 3,4 5,6");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::CurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_relative_cubic_bezier_offsets_from_current() {
        let cmds = parse_svg_path_commands("M10,10c1,1 2,2 3,3");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 10.0),
                PathCommand::CurveTo(11.0, 11.0, 12.0, 12.0, 13.0, 13.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_negative_and_comma_separated_numbers() {
        let cmds = parse_svg_path_commands("M-5.5,-2.5L+3,4");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(-5.5, -2.5),
                PathCommand::LineTo(3.0, 4.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_real_cdn_stroke_path() {
        // Verbatim fragment of cdn/kanji_animations/丁.svg — the actual
        // production input format.
        let d = "M14,24.17c2.44,0.56,6.92,0.82,9.35,0.56c18.9-1.99,39.53-5.36,60.62-6.48";
        let cmds = parse_svg_path_commands(d);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], PathCommand::MoveTo(14.0, 24.17));

        // Relative curve: offsets added to the current point (14, 24.17).
        // f64 addition carries float error (24.17+0.82), so compare with a
        // tolerance instead of exact equality.
        match cmds[1] {
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                assert!((x1 - 16.44).abs() < 1e-9);
                assert!((y1 - 24.73).abs() < 1e-9);
                assert!((x2 - 20.92).abs() < 1e-9);
                assert!((y2 - 24.99).abs() < 1e-9);
                assert!((x - 23.35).abs() < 1e-9);
                assert!((y - 24.73).abs() < 1e-9);
            },
            other => panic!("expected CurveTo, got {other:?}"),
        }
    }

    #[test]
    fn parse_path_commands_empty_and_noise_input() {
        assert!(parse_svg_path_commands("").is_empty());
        // Unknown command letters are skipped without breaking the parser.
        assert!(parse_svg_path_commands("Q 1 2 3 4").is_empty());
    }

    #[test]
    fn parse_path_commands_spaced_kanjivg_format_med() {
        // Verbatim stroke 0 of cdn/kanji_animations/医.svg — the spaced
        // KanjiVG format. The old parser aborted after the MoveTo
        // (the " c " letter read as a failed implicit-LineTo coordinate),
        // leaving the practice canvas blank for every spaced-format kanji.
        let d = "M 17.88,20.29 c 1.91,0.51 5.41,0.64 7.31,0.51 17.69,-1.18 38.69,-3.05 58.21,-3.51 3.18,-0.08 5.08,0.25 6.67,0.5";
        let cmds = parse_svg_path_commands(d);
        assert_eq!(
            cmds.len(),
            4,
            "MoveTo + three (implicit-repeat) curves, got: {cmds:?}"
        );
        assert_eq!(cmds[0], PathCommand::MoveTo(17.88, 20.29));
        // First (relative) curve: offsets added to the MoveTo point.
        let PathCommand::CurveTo(x1, y1, x2, y2, x, y) = cmds[1] else {
            panic!("expected CurveTo");
        };
        assert!((x1 - 19.79).abs() < 1e-9);
        assert!((y1 - 20.80).abs() < 1e-9);
        assert!((x2 - 23.29).abs() < 1e-9);
        assert!((y2 - 20.93).abs() < 1e-9);
        assert!((x - 25.19).abs() < 1e-9);
        assert!((y - 20.80).abs() < 1e-9);
    }

    #[test]
    fn parse_path_commands_spaced_uppercase_c_switch() {
        // Verbatim fragment of 医 stroke 6: lowercase c, then uppercase C
        // behind a space — both must parse.
        let d = "M 57.92,40.43 c 0.58,1.07 0.81,2.39 0.81,3.53 C 58.75,61.25 52.25,74 36.06,81.18";
        let cmds = parse_svg_path_commands(d);
        assert_eq!(cmds.len(), 3, "MoveTo + c-curve + C-curve, got: {cmds:?}");
    }

    #[test]
    fn parse_path_commands_smooth_cubic_reflects_previous_control() {
        // Verbatim fragment of 仰.svg: a relative smooth cubic after a
        // relative cubic. Per SVG spec the smooth command's first control
        // point reflects the previous cubic's second control point.
        let d = "M43.37,24.35c0,0.71,1.07,2.03,1.07,3.34s9-6.38,17.57-12.15";
        let cmds = parse_svg_path_commands(d);
        assert_eq!(
            cmds.len(),
            3,
            "MoveTo + curve + smooth curve, got: {cmds:?}"
        );

        // First (relative) cubic: control2 = (43.37+1.07, 24.35+2.03)
        // = (44.44, 26.38); endpoint = (44.44, 27.69).
        let PathCommand::CurveTo(_, _, x2, y2, x, y) = cmds[1] else {
            panic!("expected CurveTo");
        };
        assert!((x2 - 44.44).abs() < 1e-9 && (y2 - 26.38).abs() < 1e-9);
        let current = (x, y);

        // Smooth: given (relative) second control (9, -6.38) and endpoint
        // (17.57, -12.15); first control = reflection of (44.44, 26.38)
        // through the current point.
        let PathCommand::CurveTo(rx, ry, sx2, sy2, ex, ey) = cmds[2] else {
            panic!("expected CurveTo from S");
        };
        assert!((rx - (2.0 * current.0 - x2)).abs() < 1e-9);
        assert!((ry - (2.0 * current.1 - y2)).abs() < 1e-9);
        assert!((sx2 - (current.0 + 9.0)).abs() < 1e-9);
        assert!((sy2 - (current.1 - 6.38)).abs() < 1e-9);
        assert!((ex - (current.0 + 17.57)).abs() < 1e-9);
        assert!((ey - (current.1 - 12.15)).abs() < 1e-9);
    }

    #[test]
    fn parse_path_commands_smooth_without_previous_cubic_uses_current_point() {
        // S at the start of a path (no previous C/S): per the SVG spec
        // the first control point IS the current point.
        let cmds = parse_svg_path_commands("M10,10S20,20 30,30");
        assert_eq!(cmds.len(), 2, "got: {cmds:?}");
        assert_eq!(
            cmds[1],
            PathCommand::CurveTo(10.0, 10.0, 20.0, 20.0, 30.0, 30.0)
        );
    }

    #[test]
    fn parse_path_commands_smooth_reflection_resets_after_move_and_close() {
        // The reflection only chains through an unbroken C/S sequence:
        // M (and Z) reset the remembered control point.
        let cmds = parse_svg_path_commands("M0,0C1,1 2,2 3,3ZM5,5S6,6 7,7");
        assert_eq!(cmds.len(), 5, "M + C + Z + M + S, got: {cmds:?}");
        let PathCommand::CurveTo(rx, ry, ..) = cmds[4] else {
            panic!("expected CurveTo from S");
        };
        // Without the reset the reflection would be 2*(5,5) − (2,2) =
        // (8,8); with it the first control is the current point (5,5).
        assert!((rx - 5.0).abs() < 1e-9 && (ry - 5.0).abs() < 1e-9);
    }

    #[test]
    fn parse_stroke_paths_real_med_file_yields_all_seven_strokes() {
        // A verbatim copy of cdn/kanji_animations/医.svg, committed as a
        // fixture — the CDN store itself is gitignored, so a compile-time
        // include from there would break CI (lint/test/wasm-test jobs
        // have no art files).
        let svg = include_str!("test_kanji_med_spaced.svg");
        let strokes = parse_stroke_paths(svg);
        assert_eq!(strokes.len(), 7, "医 has 7 strokes");
        // Every stroke must produce drawable commands — a MoveTo alone
        // paints nothing.
        for (i, s) in strokes.iter().enumerate() {
            let cmds = parse_svg_path_commands(&s.d);
            assert!(
                cmds.len() >= 2,
                "stroke {i} yields only {:?} — nothing would be drawn",
                cmds
            );
        }
    }
}
