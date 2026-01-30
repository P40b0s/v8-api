use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use deno_core::{Extension, OpDecl, OpState, op2};
use font_kit::font::Font;
use tiny_skia::{Color, Paint, Pixmap, Transform};

pub struct CanvasContext 
{
    // Пиксельный буфер (RGBA)
    pub pixmap: Pixmap,
    // Текущая краска (цвет, прозрачность)
    pub paint: Paint<'static>,
    // Текущий шрифт (для fillText и measureText)
    pub font: Option<Arc<Font>>,
    pub font_size: f32,
    // Матрица трансформаций (scale, rotate)
    pub transform: Transform,
}

impl CanvasContext 
{
    pub fn new(width: u32, height: u32) -> Self {
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.anti_alias = true;

        Self {
            pixmap: Pixmap::new(width, height).unwrap(),
            paint,
            font: None,
            font_size: 12.0,
            transform: Transform::identity(),
        }
    }

    /// конвертация из Premultiplied (Skia) в Straight (Browser) иначе работать не будет
    pub fn get_unpremultiplied_rect(&self, x: i32, y: i32, w: i32, h: i32) -> Vec<u8> {
        let full_width = self.pixmap.width() as i32;
        let full_height = self.pixmap.height() as i32;
        let pixels = self.pixmap.data();
        
        // Результирующий массив: ширина * высота * 4 канала (RGBA)
        let mut result = Vec::with_capacity((w * h * 4) as usize);

        for row in 0..h {
            for col in 0..w {
                let curr_x = x + col;
                let curr_y = y + row;

                // Проверка границ, чтобы не выйти за пределы Pixmap
                if curr_x >= 0 && curr_x < full_width && curr_y >= 0 && curr_y < full_height {
                    let offset = ((curr_y * full_width + curr_x) * 4) as usize;
                    let chunk = &pixels[offset..offset + 4];
                    
                    let a = chunk[3] as f32 / 255.0;
                    if a > 0.0 {
                        result.push((chunk[0] as f32 / a).round() as u8); // R
                        result.push((chunk[1] as f32 / a).round() as u8); // G
                        result.push((chunk[2] as f32 / a).round() as u8); // B
                        result.push(chunk[3]);                            // A
                    } else {
                        result.extend_from_slice(&[0, 0, 0, 0]);
                    }
                } else {
                    // Если вышли за границы холста — отдаем прозрачный пиксель
                    result.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
        result
    }
}


pub struct CanvasManager 
{
    pub contexts: HashMap<u32, RefCell<CanvasContext>>,
    pub next_id: u32,
}

impl CanvasManager 
{
    pub fn new() -> Self 
    {
        Self 
        {
            contexts: HashMap::new(),
            next_id: 0,
        }
    }
}

pub fn init_state(state: &mut OpState) 
{
    state.put(CanvasManager 
    {
        contexts: HashMap::new(),
        next_id: 0,
    });
}

pub fn get_extension() -> (deno_core::Extension, &'static str)
{
    let shim_code = include_str!("canvas.js");
    const CREATE: OpDecl = op_canvas_create();
    const FILL: OpDecl = op_canvas_fill_rect();
    const GET_IMAGE_DATA: OpDecl = op_canvas_get_image_data();
    const TO_DATA_URL: OpDecl = op_canvas_to_data_url();
    const STYLE: OpDecl = op_canvas_set_fill_style();
    let ext = Extension 
    {
        name: "canvas_ext",
        ops: std::borrow::Cow::Borrowed(&[CREATE, FILL, GET_IMAGE_DATA, TO_DATA_URL, STYLE]),
        op_state_fn: Some(Box::new(init_state)),
        ..Default::default()
    };
    (ext, shim_code)
}


#[op2(fast)]
pub fn op_canvas_fill_rect(
    state: &mut OpState,
    #[smi] id: u32,
    x: f64, y: f64, w: f64, h: f64
) {
    let manager = state.borrow::<CanvasManager>();
    if let Some(ctx_cell) = manager.contexts.get(&id) 
    {
        let mut ctx = ctx_cell.borrow_mut();
        let rect = tiny_skia::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap();
        let transform = ctx.transform;
        let paint = ctx.paint.clone();
        
        // Рисуем на конкретном буфере
        ctx.pixmap.fill_rect(rect, &paint, transform, None);
    }
}

#[op2]
// Deno автоматически сконвертирует Vec<u8> в JS Uint8Array
fn op_canvas_get_image_data(
    state: &mut OpState, 
    #[smi] id: u32, 
    x: i32, y: i32, w: i32, h: i32
) -> Vec<u8> 
{
    let manager = state.borrow::<CanvasManager>();
    let ctx_cell = manager.contexts.get(&id).expect("Canvas not found");
    let ctx = ctx_cell.borrow();
    ctx.get_unpremultiplied_rect(x, y, w, h)
}

#[op2(fast)]
fn op_canvas_create(state: &mut OpState) -> u32 
{
    let mut manager = state.borrow_mut::<CanvasManager>();
    let id = manager.next_id;
    manager.contexts.insert(id, RefCell::new(CanvasContext::new(300, 150)));
    manager.next_id += 1;
    id
}

#[op2]
#[string]
fn op_canvas_to_data_url(state: &mut OpState, #[smi] id: u32) -> String 
{
    let manager = state.borrow::<CanvasManager>();
    let ctx_cell = manager.contexts.get(&id).expect("Canvas not found");
    let ctx = ctx_cell.borrow();

    let png_data = ctx.pixmap.encode_png().expect("Failed to encode PNG");

    let base64_str = base64::Engine::encode(&base64::engine::general_purpose::STANDARD,png_data);

    format!("data:image/png;base64,{}", base64_str)
}

#[op2(fast)]
fn op_canvas_set_fill_style(state: &mut OpState, #[smi] id: u32, #[string] color_str: String) 
{
    let manager = state.borrow::<CanvasManager>();
    if let Some(ctx_cell) = manager.contexts.get(&id) 
    {
        let mut ctx = ctx_cell.borrow_mut();
        ctx.paint.set_color(crate::helpers::colors::parse_css_color(&color_str));
    }
}