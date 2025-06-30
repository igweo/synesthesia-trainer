use gloo::timers::callback::Timeout;
use std::collections::HashMap;
use web_sys::{AudioContext, DistanceModelType, OscillatorType, PannerNode, PanningModelType};
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let current = use_state(|| 'a');
    let mastery = use_state(|| {
        let mut map = HashMap::new();
        for ch in 'a'..='z' {
            map.insert(ch, 0u32);
        }
        map
    });

    let play_tone = {
        let current = current.clone();
        Callback::from(move |_| {
            let context = AudioContext::new().unwrap();
            let gain = context.create_gain().unwrap();

            let letter = *current;
            let freq = get_frequency(letter);
            let volume = get_volume(letter);
            let angle = get_fixed_azimuth(letter);
            let now = context.current_time();

            let osc = context.create_oscillator().unwrap();
            osc.frequency().set_value(freq);
            osc.set_type(OscillatorType::Triangle);

            let panner = context.create_panner().unwrap();
            panner.set_panning_model(PanningModelType::Hrtf);
            panner.set_distance_model(DistanceModelType::Inverse);
            panner.set_position(angle.cos().into(), 0.0, angle.sin().into());

            gain.gain().set_value_at_time(0.0, now).unwrap();
            gain.gain()
                .linear_ramp_to_value_at_time(volume, now + 0.05)
                .unwrap();
            gain.gain()
                .linear_ramp_to_value_at_time(0.0, now + 0.5)
                .unwrap();

            osc.connect_with_audio_node(&gain).unwrap();
            gain.connect_with_audio_node(&panner).unwrap();
            panner
                .connect_with_audio_node(&context.destination())
                .unwrap();

            osc.start().unwrap();
            Timeout::new(500, move || {
                osc.stop().ok();
            })
            .forget();
        })
    };

    let next_letter = {
        let current = current.clone();
        let mastery = mastery.clone();
        Callback::from(move |_| {
            let mut new_map = (*mastery).clone();
            if let Some(score) = new_map.get_mut(&*current) {
                *score += 1;
            }
            let mut scores: Vec<_> = new_map.iter().collect();
            scores.sort_by_key(|(_, score)| *score);
            if let Some(&(next, _)) = scores.first() {
                current.set(*next);
            }
            mastery.set(new_map);
        })
    };

    html! {
        <div class="trainer" onclick={play_tone}>
            <div class="background" style={format!("background-color: {};", get_color(*current))}>
                <div class="letter">{ *current }</div>
            </div>
            <button onclick={next_letter}>{ "Next" }</button>
        </div>
    }
}

fn get_fixed_azimuth(letter: char) -> f32 {
    // 26 letters evenly spaced around a circle
    let index = letter.to_ascii_lowercase() as usize - 'a' as usize;
    let degrees = (index as f32 / 26.0) * 360.0;
    degrees.to_radians()
}

fn get_frequency(letter: char) -> f32 {
    let brightness = get_brightness(letter);
    220.0 * (8.0f32).powf(brightness)
}

fn get_volume(letter: char) -> f32 {
    get_saturation(letter)
}

fn get_color(letter: char) -> &'static str {
    match letter {
        'a' => "#FFA3E2",
        'b' => "#6699FF",
        'c' => "#00EBEB",
        'd' => "#FF6600",
        'e' => "#A9E5A9",
        'f' => "#ff24d3",
        'g' => "#33FF33",
        'h' => "#DD2782",
        'i' => "#BAA3FF",
        'j' => "#00CC66",
        'k' => "#CCB300",
        'l' => "#FF6B6B",
        'm' => "#9005b3",
        'n' => "#a6f2f2",
        'o' => "#FFCB94",
        'p' => "#c905ff",
        'q' => "#2BAB8B",
        'r' => "#FF0000",
        's' => "#de895e",
        't' => "#00FFCC",
        'u' => "#3f4da6",
        'v' => "#D557FF",
        'w' => "#FFCC33",
        'x' => "#B2DF2A",
        'y' => "#FFFF00",
        'z' => "#7898D9",
        _ => "#FFFFFF",
    }
}

fn hex_to_hsb(hex: &str) -> (f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (0.0, 0.0, 1.0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        ((b - r) / delta) + 2.0
    } else {
        ((r - g) / delta) + 4.0
    } * 60.0;

    let hue = (hue + 360.0) % 360.0;
    let sat = if max == 0.0 { 0.0 } else { delta / max };
    let bright = max;

    (hue / 360.0, sat, bright)
}

fn get_brightness(letter: char) -> f32 {
    let color = get_color(letter);
    let (_, _, b) = hex_to_hsb(color);
    b
}

fn get_saturation(letter: char) -> f32 {
    let color = get_color(letter);
    let (_, s, _) = hex_to_hsb(color);
    s
}

fn main() {
    yew::Renderer::<App>::new().render();
}
