use glium;

/// Convenience method to compile the shader program used by the renderer.
pub fn get_program(display: &glium::Display) -> glium::Program {
  let v_shader = r#"
    #version 130

    uniform mat4 proj_mat;

    in vec2 pos;
    in vec2 tex_coords;
    in vec4 col; 

    out vec2 v_tex_coords;
    out vec4 v_col;

    void main() {
      v_col = col;
      v_tex_coords = tex_coords;
      gl_Position = proj_mat*vec4(pos, 0.0, 1.0);
    }
  "#;

  let f_shader = r#"
    #version 130

    uniform sampler2D tex;

    in vec4 v_col;
    in vec2 v_tex_coords;

    out vec4 color;

    void main() {
      color = texture(tex, v_tex_coords);
    }
  "#;
  glium::Program::from_source(display, v_shader, f_shader, None).unwrap()
}
