// shim.js
(function() {
    const { ops } = Deno.core;

    class CanvasRenderingContext2D 
    {
        constructor(id) 
        {
            this.id = id;
            this.width = 300;
            this.height = 150;
            this._fillStyle = '#000000';
        }
        
        fillRect(x, y, w, h) 
        {
            ops.op_canvas_fill_rect(this.id, 
                Math.fround(x), 
                Math.fround(y), 
                Math.fround(w), 
                Math.fround(h));
        }
        
        set fillStyle(color) 
        {
            this._fillStyle = color;
            ops.op_canvas_set_fill_style(this.id, color);
        }
        get fillStyle() 
        {
            return this._fillStyle;
        }

        getImageData(x, y, w, h) 
        {
            const rawBytes = ops.op_canvas_get_image_data(this.id, x, y, w, h);
            //ImageData требует Uint8ClampedArray. 
            const clampedArray = new Uint8ClampedArray(rawBytes.buffer);

            // 3. Возвращаем стандартный объект ImageData
            // { data: [...], width: 8, height: 8 }
            return {
                data: clampedArray,
                width: w,
                height: h,
                colorSpace: 'srgb'
            };
        }
        toDataURL(type = "image/png", encoderOptions) 
        {
            // отдаем только PNG,
            return ops.op_canvas_to_data_url(this.id);
        }
        getContext(type, options) 
        {
            if (type === '2d') 
            {
                return new CanvasRenderingContext2D(this.id);
            }
        }
    }

    class HTMLCanvasElement 
    {
        constructor() 
        {
            this.style = {};
            this.width = 300;
            this.height = 150;
        }
        getContext(type) 
        {
            if (type === '2d') return new CanvasRenderingContext2D(this);
            throw new Error("WebGL not implemented yet");
        }
        toDataURL() 
        {
            return ops.op_canvas_to_data_url();
        }

    }

    // Эмуляция DOM
    globalThis.window = globalThis;
    globalThis.document = {
        createElement: (tag) => {
            if (tag === 'canvas') {
                const id = ops.op_canvas_create();
                return {
                    getContext: () => new CanvasRenderingContext2D(id),
                    width: 300,
                    height: 150
                };
            }
        }
    };
})();