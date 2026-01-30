use tiny_skia::Color;

pub fn parse_css_color(color_str: &str) -> Color 
{
    let s = color_str.to_lowercase().replace(" ", "");

    if s.starts_with("rgb") 
    {
        let values: Vec<&str> = s
            .trim_matches(|c| c == 'r' || c == 'g' || c == 'b' || c == 'a' || c == '(' || c == ')')
            .split(',')
            .collect();

        if values.len() >= 3 
        {
            let r = values[0].parse::<f32>().unwrap_or(0.0) / 255.0;
            let g = values[1].parse::<f32>().unwrap_or(0.0) / 255.0;
            let b = values[2].parse::<f32>().unwrap_or(0.0) / 255.0;
            let a = if values.len() == 4 {
                values[3].parse::<f32>().unwrap_or(1.0)
            } 
            else 
            {
                1.0
            };
            // tiny-skia требует Premultiplied Color для рисования!
            return Color::from_rgba(r * a, g * a, b * a, a).expect("Invalid color");
        }
    }

    // 2. Обработка HEX #RRGGBB
    if s.starts_with('#') 
    {
        if let Ok(r) = u8::from_str_radix(&s[1..3], 16) 
        {
            if let Ok(g) = u8::from_str_radix(&s[3..5], 16) 
            {
                if let Ok(b) = u8::from_str_radix(&s[5..7], 16) 
                {
                    return Color::from_rgba8(r, g, b, 255);
                    //return Color::from_rgba8(r, g, b, 255).premultiply();
                }
            }
        }
    }

    Color::BLACK // Значение по умолчанию
}