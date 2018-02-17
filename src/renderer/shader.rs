use glium;

/// Convenience method to compile the shader program used by the renderer.
pub fn get_program<F: glium::backend::Facade>(display: &F) -> glium::Program {
    let v_shader = r#"
    #version 120

    uniform mat4 proj_mat;

    attribute vec2 pos;
    attribute vec2 tex_coords;
    attribute vec4 col; 

    varying vec2 v_tex_coords;
    varying vec4 v_col;

    void main() {
      v_col = col;
      v_tex_coords = tex_coords;
      gl_Position = proj_mat*vec4(pos, 0.0, 1.0);
    }
  "#;

    let f_shader = r#"
    #version 120

    uniform sampler2D tex;

    // If we're rendering a font, we only care about the r value of the tex.
    // Otherwise, we care about the colour. Will be 1 if we're rendering a font.
    uniform int is_font;

    varying vec4 v_col;
    varying vec2 v_tex_coords;

    void main() {
      if (is_font > 0) {
        gl_FragColor = vec4(v_col.rgb, texture2D(tex, v_tex_coords).r);
      }
      else {
        vec4 pixel = texture2D(tex, v_tex_coords);
        gl_FragColor = vec4(pixel.r * v_col.r, 
                     pixel.g * v_col.g, 
                     pixel.b * v_col.b, 
                     pixel.a * v_col.a);
      }
    }
  "#;
    glium::Program::from_source(display, v_shader, f_shader, None).unwrap()
}
