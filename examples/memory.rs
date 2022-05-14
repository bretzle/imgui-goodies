use imgoodies::{memory::MemoryEditor, Framework};
use imgui::Ui;
use std::mem::MaybeUninit;

const SIZE: usize = 0x40000;

struct State {
    editor: MemoryEditor,
    data: Box<[u8; SIZE]>,
}

fn main() {
    let mut state = State {
        editor: MemoryEditor::new(),
        data: unsafe { Box::new(MaybeUninit::uninit().assume_init()) },
    };

    // state.data
    unsafe {
        std::ptr::copy(b"Hello World!", state.data.as_mut_ptr().cast(), 1);
    }

    Framework::new("Memory Editor demo", state).run(draw);
}

fn draw(ui: &mut Ui, state: &mut State) {
    state.editor.draw_window(ui, state.data.as_mut_slice());
}
