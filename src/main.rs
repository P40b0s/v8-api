use deno_core::{Extension, JsRuntime, OpDecl, OpState, RuntimeOptions, extension, op2};
use font_kit::canvas;
use tiny_skia::{Paint, Rect, Pixmap, FillRule, Transform};
use std::{rc::Rc};
use std::cell::RefCell;
use anyhow::{Context, Result};

use crate::api::canvas::CanvasManager;
mod api;
mod helpers;


#[tokio::main]
async fn main() -> Result<()>
{
    let (canv_ext, canvas_code) = api::canvas::get_extension();
    let mut runtime = JsRuntime::new(RuntimeOptions 
    {
        extensions: vec![canv_ext],
        ..Default::default()
    });
    runtime.execute_script("shim.js", canvas_code)
        .context("Error when run script shim.js")?;
    // имитируем внешний скрипт
     let js_code = r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        ctx.fillRect(0, 0, 100, 100);
    "#;

    runtime.execute_script("logic.js", js_code).unwrap();
    
    // для теста сохраняем первый канвас из менеджера
    let ref_state = runtime.op_state();
    let state = ref_state.borrow();
    let canvas_state = state.borrow::<CanvasManager>();
    canvas_state.contexts.values().next().unwrap().borrow().pixmap.save_png("result.png").unwrap();
    
    println!("Canvas отрендерен моментально без браузера!");
    Ok(())
}