use glium;
use glium::backend::glutin_backend::GlutinFacade;

/// Convenience method to compile the shader program used by the renderer.
pub fn get_program(display: &GlutinFacade) -> glium::Program {
  let v_shader = r#"
    #version 130

  uniform mat4 proj_mat;

    in vec2 pos;
    in vec4 col;

    out vec4 v_col;

    void main() {
      v_col = col;
      gl_Position = proj_mat*vec4(pos, 0.0, 1.0);
    }
  "#;

  let f_shader = r#"
    #version 130

    in vec4 v_col;

    out vec4 color;

    void main() {
      color = v_col;
    }
  "#;
  glium::Program::from_source(display, v_shader, f_shader, None).unwrap()
}
