use gemstone::mem::zero;
use imgui::{
    sys::{
        igGetColorU32Col, igGetFrameHeightWithSpacing, igGetTextLineHeight,
        igGetTextLineHeightWithSpacing, igSetCursorPosX, igSetWindowSizeVec2,
    },
    ComboBoxFlags, Condition, InputTextCallback, InputTextCallbackHandler, InputTextFlags, Key,
    ListClipper, MouseButton, StyleColor, StyleVar, Ui, WindowFlags, WindowHoveredFlags,
};
use std::mem::{size_of, transmute};

#[derive(PartialEq, Clone, Copy)]
enum DataFormat {
    Bin,
    Dec,
    Hex,
}

#[derive(PartialEq, Clone, Copy)]
enum DataType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl DataType {
    const ALL: [Self; 10] = [
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::F32,
        Self::F64,
    ];

    fn size(&self) -> usize {
        match self {
            DataType::I8 => size_of::<i8>(),
            DataType::I16 => size_of::<i16>(),
            DataType::I32 => size_of::<i32>(),
            DataType::I64 => size_of::<i64>(),
            DataType::U8 => size_of::<u8>(),
            DataType::U16 => size_of::<u16>(),
            DataType::U32 => size_of::<u32>(),
            DataType::U64 => size_of::<u64>(),
            DataType::F32 => size_of::<f32>(),
            DataType::F64 => size_of::<f64>(),
        }
    }

    fn desc(&self) -> &'static str {
        match self {
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
        }
    }
}

type ReadFn<T> = fn(data: &T, off: usize);
type WriteFn<T> = fn(data: &mut T, off: usize, val: u8);
type HighligtFn<T> = fn(data: &T, off: usize);

pub struct MemoryEditor {
    contents_width_changed: bool,
    data_preview_adr: usize,
    data_editing_addr: usize,
    data_editing_take_focus: bool,
    data_input_buf: String,
    addr_input_buf: String,
    goto_addr: usize,
    highlight_min: usize,
    highlight_max: usize,
    preview_endianess: i32,
    preview_data_type: DataType,

    // Settings
    open: bool,
    read_only: bool,
    cols: i32,
    show_options: bool,
    show_data_preview: bool,
    show_hexii: bool,
    show_ascii: bool,
    grey_out_zeros: bool,
    uppercase_hex: bool,
    mid_cols_count: usize,
    addr_digits_count: usize,
    footer_extra_height: f32,
    highlight_color: [f32; 4],
    read_fn: Option<ReadFn<[u8]>>,
    write_fn: Option<WriteFn<[u8]>>,
    highlight_fn: Option<HighligtFn<[u8]>>,
}

impl MemoryEditor {
    pub fn new() -> Self {
        Self {
            contents_width_changed: false,
            data_preview_adr: usize::MAX,
            data_editing_addr: usize::MAX,
            data_editing_take_focus: false,
            data_input_buf: String::with_capacity(32),
            addr_input_buf: String::with_capacity(32),
            goto_addr: usize::MAX,
            highlight_min: usize::MAX,
            highlight_max: usize::MAX,
            preview_endianess: 0,
            preview_data_type: DataType::I32,
            open: true,
            read_only: false,
            cols: 16,
            show_options: true,
            show_data_preview: false,
            show_hexii: false,
            show_ascii: true,
            grey_out_zeros: true,
            uppercase_hex: true,
            mid_cols_count: 8,
            addr_digits_count: 0,
            footer_extra_height: 0.0,
            highlight_color: [255.0, 255.0, 255.0, 50.0],
            read_fn: None,
            write_fn: None,
            highlight_fn: None,
        }
    }

    pub fn open(&self) -> bool {
        self.open
    }
}

struct Sizes {
    addr_digit_count: usize,
    line_height: f32,
    glyph_width: f32,
    hex_cell_width: f32,
    spacing_between_mid_cols: f32,
    pos_hex_start: f32,
    pos_hex_end: f32,
    pos_ascii_start: f32,
    pos_ascii_end: f32,
    window_width: f32,
}

impl MemoryEditor {
    unsafe fn calc_sizes(&self, ui: &Ui, mem_size: usize, base_display_addr: usize) -> Sizes {
        let style = ui.style();
        let mut s: Sizes = zero();
        s.addr_digit_count = self.addr_digits_count;
        if s.addr_digit_count == 0 {
            let mut n = base_display_addr + mem_size - 1;
            while n > 0 {
                s.addr_digit_count += 1;
                n >>= 4;
            }
        }
        s.line_height = igGetTextLineHeight();
        s.glyph_width = ui.calc_text_size("F")[0] + 1.0; // We assume the font is mono-space
        s.hex_cell_width = (s.glyph_width * 2.5) as i32 as f32; // "FF " we include trailing space in the width to easily catch clicks everywhere
        s.spacing_between_mid_cols = (s.hex_cell_width * 0.25) as i32 as f32; // Every OptMidColsCount columns we add a bit of extra spacing
        s.pos_hex_start = (s.addr_digit_count + 2) as f32 * s.glyph_width;
        s.pos_hex_end = s.pos_hex_start + (s.hex_cell_width * self.cols as f32);
        s.pos_ascii_start = s.pos_hex_end;
        s.pos_ascii_end = s.pos_hex_end;
        if self.show_ascii {
            s.pos_ascii_start = s.pos_hex_end + s.glyph_width * 1.0;
            if self.mid_cols_count > 0 {
                s.pos_ascii_start += ((self.cols as usize + self.mid_cols_count - 1)
                    / self.mid_cols_count as usize) as f32
                    * s.spacing_between_mid_cols;
            }
            s.pos_ascii_end = s.pos_ascii_start + self.cols as f32 * s.glyph_width;
        }
        s.window_width =
            s.pos_ascii_end + style.scrollbar_size + style.window_padding[0] * 2.0 + s.glyph_width;
        s
    }

    pub fn draw_window(&mut self, ui: &Ui, data: &mut [u8]) {
        let base_display_addr = 0x0000;

        let mem_size = data.len();

        let mut size = unsafe { self.calc_sizes(ui, mem_size, base_display_addr) };

        self.open = true;

        ui.window("Memory Editor")
            .size(
                [size.window_width, size.window_width * 0.6],
                Condition::FirstUseEver,
            )
            .size_constraints([0.0, 0.0], [size.window_width, f32::MAX])
            .opened(unsafe { transmute(&mut self.open) })
            .flags(WindowFlags::NO_SCROLLBAR)
            .build(|| {
                if ui.is_window_hovered_with_flags(WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS)
                    && ui.is_mouse_released(MouseButton::Right)
                {
                    ui.open_popup("context")
                }

                unsafe {
                    self.draw_contents(ui, data, mem_size, base_display_addr);
                }

                if self.contents_width_changed {
                    unsafe {
                        size = self.calc_sizes(ui, mem_size, base_display_addr);
                        igSetWindowSizeVec2([size.window_width, ui.window_size()[1]].into(), 0);
                    }
                }
            });
    }

    unsafe fn draw_contents(
        &mut self,
        ui: &Ui,
        data: &mut [u8],
        mem_size: usize,
        base_display_addr: usize,
    ) {
        if self.cols < 1 {
            self.cols = 1;
        }

        let s = self.calc_sizes(ui, mem_size, base_display_addr);
        let style = ui.style();

        // We begin into our scrolling region with the 'ImGuiWindowFlags_NoMove' in order to prevent click from moving the window.
        // This is used as a facility since our main click detection code doesn't assign an ActiveId so the click would normally be caught as a window-move.
        let height_separator = style.item_spacing[1];
        let mut footer_height = self.footer_extra_height;
        if self.show_options {
            footer_height += height_separator + igGetFrameHeightWithSpacing() * 1.0
        }
        if self.show_data_preview {
            footer_height += height_separator
                + igGetFrameHeightWithSpacing() * 1.0
                + igGetTextLineHeightWithSpacing() * 3.0;
        }

        let mut data_next = false;
        let mut data_editing_addr_next = usize::MAX;
        if self.data_editing_addr != usize::MAX {
            if ui.is_key_pressed(Key::UpArrow)
                && self.data_editing_addr as isize >= self.cols as isize
            {
                data_editing_addr_next = self.data_editing_addr - self.cols as usize;
            } else if ui.is_key_pressed(Key::DownArrow)
                && (self.data_editing_addr as isize) < ((mem_size - self.cols as usize) as isize)
            {
                data_editing_addr_next = self.data_editing_addr + self.cols as usize;
            } else if ui.is_key_pressed(Key::LeftArrow)
                && (self.data_editing_addr as isize) > 0isize
            {
                data_editing_addr_next = self.data_editing_addr - 1;
            } else if ui.is_key_pressed(Key::RightArrow)
                && (self.data_editing_addr as isize) < ((mem_size - 1) as isize)
            {
                data_editing_addr_next = self.data_editing_addr + 1;
            }
        }

        ui.child_window("##scrolling")
            .size([0.0, -footer_height])
            .border(false)
            .flags(WindowFlags::NO_MOVE | WindowFlags::NO_NAV)
            .build(|| {
                let draw_list = ui.get_window_draw_list();

                let _t1 = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                let _t2 = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

                // We are not really using the clipper API correctly here, because we rely on visible_start_addr/visible_end_addr for our scrolling function.
                let line_total_count =
                    ((mem_size + self.cols as usize - 1) / self.cols as usize) as i32;
                let _clipper = ListClipper::new(line_total_count).items_height(s.line_height);
                let mut clipper = _clipper.begin(ui);

                if self.read_only || self.data_editing_addr >= mem_size {
                    self.data_editing_addr = usize::MAX;
                }
                if self.data_preview_adr >= mem_size {
                    self.data_preview_adr = usize::MAX;
                }

                let preview_data_type_size = if self.show_data_preview {
                    self.preview_data_type.size()
                } else {
                    0
                };

                // Draw vertical separator
                let window_pos = ui.window_pos();
                if self.show_ascii {
                    draw_list
                        .add_line(
                            [
                                window_pos[0] + s.pos_ascii_start - s.glyph_width,
                                window_pos[1],
                            ],
                            [
                                window_pos[0] + s.pos_ascii_start - s.glyph_width,
                                window_pos[1] + 9999.0,
                            ],
                            igGetColorU32Col(StyleColor::Border as i32, 1.0),
                        )
                        .build();
                }

                let color_text = igGetColorU32Col(StyleColor::Text as i32, 1.0);
                let color_disabled = igGetColorU32Col(StyleColor::TextDisabled as i32, 1.0);

                while clipper.step() {
                    for line_i in clipper.display_start()..clipper.display_end() {
                        let mut addr = line_i as usize * self.cols as usize;
                        ui.text(format!("{:04X}", base_display_addr + addr));

                        // Draw hexadecimal
                        let mut n = 0;
                        while n < self.cols && addr < mem_size {
                            let mut byte_pos_x = s.pos_hex_start + s.hex_cell_width * n as f32;
                            if self.mid_cols_count > 0 {
                                byte_pos_x += (n as usize / self.mid_cols_count) as f32
                                    * s.spacing_between_mid_cols;
                            }
                            ui.same_line_with_pos(byte_pos_x);

                            // Draw Highlight
                            let is_highlight_from_user_range =
                                addr >= self.highlight_min && addr < self.highlight_max;
                            let is_highlight_from_user_func = false; // TODO: use highlight_fn
                            let is_highlight_from_preview = addr >= self.data_preview_adr
                                && addr < self.data_preview_adr + preview_data_type_size;
                            if is_highlight_from_user_range
                                || is_highlight_from_user_func
                                || is_highlight_from_preview
                            {
                                let pos = ui.cursor_screen_pos();
                                let mut highlight_width = s.glyph_width * 2.0;
                                let is_next_byte_highlighted = (addr + 1 < mem_size)
                                    && (self.highlight_max != usize::MAX
                                        && addr + 1 < self.highlight_max);
                                // TODO || (self.HighlightFn && HighlightFn(mem_data, addr + 1)));
                                if is_next_byte_highlighted || (n + 1 == self.cols) {
                                    highlight_width = s.hex_cell_width;
                                    if self.mid_cols_count > 0
                                        && n > 0
                                        && (n + 1) < self.cols
                                        && ((n + 1) % self.mid_cols_count as i32) == 0
                                    {
                                        highlight_width += s.spacing_between_mid_cols;
                                    }
                                }
                                draw_list
                                    .add_rect(
                                        pos,
                                        [pos[0] + highlight_width, pos[1] + s.line_height],
                                        self.highlight_color,
                                    )
                                    .filled(true)
                                    .build();
                            }

                            if self.data_editing_addr == addr {
                                // Display text input on current byte
                                let mut data_write = false;
                                // ImGui::PushID((void*)addr);
                                let _t4 = ui.push_id_usize(addr);
                                if self.data_editing_take_focus {
                                    ui.set_keyboard_focus_here_with_offset(
                                        imgui::FocusedWidget::Offset(0),
                                    );
                                    self.addr_input_buf =
                                        format!("{:04X}", base_display_addr + addr);
                                    self.data_input_buf = format!("{:02X}", data[addr]);
                                }
                                struct UserData {
                                    current_buf_overwrite: String, // Input
                                    cursor_pos: i32,               // Output
                                }

                                impl InputTextCallbackHandler for &mut UserData {
                                    fn on_always(&mut self, mut data: imgui::TextCallbackData) {
                                        if !data.has_selection() {
                                            self.cursor_pos = data.cursor_pos() as i32;
                                        }
                                        if data.selection() == (0..data.str().len()) {
                                            // When not editing a byte, always refresh its InputText content pulled from underlying memory data
                                            // (this is a bit tricky, since InputText technically "owns" the master copy of the buffer we edit it in there)
                                            data.remove_chars(0, data.str().len());
                                            data.insert_chars(0, &self.current_buf_overwrite);
                                            *data.selection_start_mut() = 0;
                                            *data.selection_end_mut() = 2;
                                            data.set_cursor_pos(0);
                                        }
                                    }
                                }

                                let mut user_data = UserData {
                                    current_buf_overwrite: format!("{:02X}", data[addr]), // TODO: read_fn
                                    cursor_pos: -1,
                                };
                                let flags = InputTextFlags::CHARS_HEXADECIMAL
                                    | InputTextFlags::ENTER_RETURNS_TRUE
                                    | InputTextFlags::AUTO_SELECT_ALL
                                    | InputTextFlags::NO_HORIZONTAL_SCROLL
                                    | InputTextFlags::CALLBACK_ALWAYS
                                    | InputTextFlags::ALWAYS_OVERWRITE;

                                ui.set_next_item_width(s.glyph_width * 2.0);
                                if ui
                                    .input_text("##data", &mut self.data_input_buf)
                                    .flags(flags)
                                    .callback(InputTextCallback::ALWAYS, &mut user_data)
                                    .build()
                                {
                                    data_write = true;
                                    data_next = true;
                                } else if !self.data_editing_take_focus && !ui.is_item_active() {
                                    self.data_editing_addr = usize::MAX;
                                    data_editing_addr_next = usize::MAX;
                                }

                                self.data_editing_take_focus = false;
                                if user_data.cursor_pos >= 2 {
                                    data_write = true;
                                    data_next = true;
                                }
                                if data_editing_addr_next != usize::MAX {
                                    data_write = false;
                                    data_next = false;
                                }
                                if data_write {
                                    if let Ok(val) = u32::from_str_radix(&self.data_input_buf, 16) {
                                        match self.write_fn {
                                            Some(_) => todo!(),
                                            None => data[addr] = val as u8,
                                        }
                                    }
                                }
                                _t4.pop();
                            } else {
                                let byte = data[addr]; // TODO: hook into read_fn

                                if self.show_hexii {
                                    if byte >= 32 && byte < 128 {
                                        ui.text(format!(
                                            ".{} ",
                                            char::from_u32_unchecked(byte as u32)
                                        ));
                                    } else if byte == 0xFF && self.grey_out_zeros {
                                        ui.text_disabled("## ");
                                    } else if byte == 0x00 {
                                        ui.text("   ");
                                    } else {
                                        ui.text(format!("{byte:02X}"));
                                    }
                                } else {
                                    if byte == 0 && self.grey_out_zeros {
                                        ui.text_disabled("00 ");
                                    } else {
                                        ui.text(format!("{byte:02X}"));
                                    }
                                }

                                if !self.read_only
                                    && ui.is_item_hovered()
                                    && ui.is_mouse_clicked(MouseButton::Left)
                                {
                                    self.data_editing_take_focus = true;
                                    data_editing_addr_next = addr;
                                }
                            }

                            n += 1;
                            addr += 1;
                        }

                        if self.show_ascii {
                            ui.same_line_with_pos(s.pos_ascii_start);
                            let mut pos = ui.cursor_screen_pos();
                            addr = line_i as usize * self.cols as usize;
                            let t3 = ui.push_id_int(line_i);

                            if ui.invisible_button(
                                "ascii",
                                [s.pos_ascii_end - s.pos_ascii_start, s.line_height],
                            ) {
                                let x = addr
                                    + ((ui.io().mouse_pos[0] - pos[0]) / s.glyph_width) as usize;
                                self.data_editing_addr = x;
                                self.data_preview_adr = x;
                                self.data_editing_take_focus = true;
                            }
                            t3.pop();

                            let mut n = 0;
                            while n < self.cols && addr < mem_size {
                                if addr == self.data_editing_addr {
                                    draw_list
                                        .add_rect(
                                            pos,
                                            [pos[0] + s.glyph_width, pos[1] + s.line_height],
                                            igGetColorU32Col(StyleColor::FrameBg as i32, 1.0),
                                        )
                                        .filled(true)
                                        .build();
                                    draw_list
                                        .add_rect(
                                            pos,
                                            [pos[0] + s.glyph_width, pos[1] + s.line_height],
                                            igGetColorU32Col(
                                                StyleColor::TextSelectedBg as i32,
                                                1.0,
                                            ),
                                        )
                                        .filled(true)
                                        .build();
                                }

                                let c = data[addr]; // TODO: read_fn
                                let disp = if c < 32 || c >= 128 {
                                    b"."
                                } else {
                                    std::slice::from_raw_parts(&c, 1)
                                };

                                draw_list.add_text(
                                    pos,
                                    if disp[0] == c {
                                        color_text
                                    } else {
                                        color_disabled
                                    },
                                    std::str::from_utf8_unchecked(disp),
                                );

                                pos[0] += s.glyph_width;
                                n += 1;
                                addr += 1;
                            }
                        }
                    }
                }
            });

        // Notify the main window of our ideal child content size (FIXME: we are missing an API to get the contents size from the child)
        igSetCursorPosX(s.window_width);

        if data_next && self.data_editing_addr + 1 < mem_size {
            self.data_editing_addr = self.data_editing_addr + 1;
            self.data_preview_adr = self.data_editing_addr + 1;
            self.data_editing_take_focus = true;
        } else if data_editing_addr_next != usize::MAX {
            self.data_editing_addr = data_editing_addr_next;
            self.data_preview_adr = data_editing_addr_next;
            self.data_editing_take_focus = true;
        }

        let lock_show_data_preview = self.show_data_preview;
        if self.show_options {
            ui.separator();
            self.draw_options_line(ui, &s, data, mem_size, base_display_addr);
        }

        if lock_show_data_preview {
            ui.separator();
            self.draw_preview_line(ui, &s, data, mem_size);
        }
    }

    unsafe fn draw_options_line(
        &mut self,
        ui: &Ui,
        s: &Sizes,
        _data: &mut [u8],
        mem_size: usize,
        base_display_addr: usize,
    ) {
        let style = ui.style();

        if ui.button("Options") {
            ui.open_popup("context");
        }
        ui.popup("context", || {
            ui.set_next_item_width(s.glyph_width * 7.0 + style.frame_padding[0] * 2.0);
            // TODO: should have speed of 0.2
            if ui
                .slider_config("##cols", 4, 32)
                .display_format("%d cols")
                .build(&mut self.cols)
            {
                self.contents_width_changed = true;
                if self.cols < 1 {
                    self.cols = 1;
                }
            }

            ui.checkbox("Show Data Preview", &mut self.show_data_preview);
            ui.checkbox("Show HexII", &mut self.show_hexii);
            if ui.checkbox("Show Ascii", &mut self.show_ascii) {
                self.contents_width_changed = true;
            }
            ui.checkbox("Grey out zeroes", &mut self.grey_out_zeros);
            ui.checkbox("Uppercase Hex", &mut self.uppercase_hex);
        });

        ui.same_line();
        ui.text(format!(
            "Range {:04X}..{:04X}",
            base_display_addr,
            base_display_addr + mem_size - 1
        ));
        ui.same_line();
        ui.set_next_item_width(
            (s.addr_digit_count + 1) as f32 * s.glyph_width + style.frame_padding[0] * 2.0,
        );
        if ui
            .input_text("##addr", &mut self.addr_input_buf)
            .flags(InputTextFlags::CHARS_HEXADECIMAL | InputTextFlags::ENTER_RETURNS_TRUE)
            .build()
        {
            if let Ok(x) = usize::from_str_radix(&self.addr_input_buf, 16) {
                self.goto_addr = x - base_display_addr;
                self.highlight_min = usize::MAX;
                self.highlight_max = usize::MAX;
            }
        }

        if self.goto_addr != usize::MAX {
            if self.goto_addr < mem_size {
                ui.child_window("##scrolling").build(|| {
                    ui.set_scroll_from_pos_y(
                        ui.cursor_start_pos()[1]
                            + (self.goto_addr / self.cols as usize) as f32 * ui.text_line_height(),
                    );
                });
                self.data_editing_addr = self.goto_addr;
                self.data_preview_adr = self.goto_addr;
                self.data_editing_take_focus = true;
            }
            self.goto_addr = usize::MAX;
        }
    }

    unsafe fn draw_preview_line(&mut self, ui: &Ui, s: &Sizes, data: &mut [u8], mem_size: usize) {
        let style = ui.style();

        ui.align_text_to_frame_padding();
        ui.text("Preview as:");
        ui.same_line();
        ui.set_next_item_width(
            (s.glyph_width * 10.0) + style.frame_padding[0] * 2.0 + style.item_inner_spacing[0],
        );
        if let Some(t) = ui.begin_combo_with_flags(
            "##combo_type",
            self.preview_data_type.desc(),
            ComboBoxFlags::HEIGHT_LARGEST,
        ) {
            for typ in DataType::ALL {
                if ui
                    .selectable_config(typ.desc())
                    .selected(self.preview_data_type == typ)
                    .build()
                {
                    self.preview_data_type = typ;
                }
            }
        }
        // TODO: allow changing endianess
        // ui.same_line();
        // ui.set_next_item_width(
        //     (s.GlyphWidth * 6.0) + style.frame_padding[0] * 2.0 + style.item_inner_spacing[0],
        // );
        // ui.combo("##combo_endianess", &mut self.PreviewEndianess, )

        let x = s.glyph_width * 6.0;
        let has_value = self.data_preview_adr != usize::MAX;
        let mut buf = String::new();

        if has_value {
            self.draw_preview_data(
                self.data_preview_adr,
                data,
                mem_size,
                self.preview_data_type,
                DataFormat::Dec,
                &mut buf,
            );
        }
        ui.text("Dec");
        ui.same_line_with_pos(x);
        ui.text(if has_value { &buf } else { "N/A" });
        if has_value {
            self.draw_preview_data(
                self.data_preview_adr,
                data,
                mem_size,
                self.preview_data_type,
                DataFormat::Hex,
                &mut buf,
            );
        }
        ui.text("Hex");
        ui.same_line_with_pos(x);
        ui.text(if has_value { &buf } else { "N/A" });
        if has_value {
            self.draw_preview_data(
                self.data_preview_adr,
                data,
                mem_size,
                self.preview_data_type,
                DataFormat::Bin,
                &mut buf,
            );
        }
        ui.text("Bin");
        ui.same_line_with_pos(x);
        ui.text(if has_value { &buf } else { "N/A" });
    }

    unsafe fn draw_preview_data(
        &self,
        addr: usize,
        data: &mut [u8],
        mem_size: usize,
        data_type: DataType,
        data_format: DataFormat,
        out: &mut String,
    ) {
        let mut buf = [0; 8];
        let elem_size = data_type.size();
        let size = if addr + elem_size > mem_size {
            mem_size - addr
        } else {
            elem_size
        };

        match self.read_fn {
            Some(_) => todo!(),
            None => std::ptr::copy(data.as_ptr().add(addr), buf.as_mut_ptr(), size),
        }

        out.clear();

        if data_format == DataFormat::Bin {
            let mut binbuf = [0; 8];
            std::ptr::copy(buf.as_ptr(), binbuf.as_mut_ptr(), size);
            write!(out, "{}", format_binary(&binbuf, size * 8)).unwrap();
            return;
        }

        use gemstone::mem::MemValue;
        use std::fmt::Write;

        match data_type {
            DataType::I8 => match data_format {
                DataFormat::Dec => write!(out, "{}", i8::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", i8::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::I16 => match data_format {
                DataFormat::Dec => write!(out, "{}", i16::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", i16::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::I32 => match data_format {
                DataFormat::Dec => write!(out, "{}", i32::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", i32::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::I64 => match data_format {
                DataFormat::Dec => write!(out, "{}", i64::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", i64::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::U8 => match data_format {
                DataFormat::Dec => write!(out, "{}", u8::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", u8::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::U16 => match data_format {
                DataFormat::Dec => write!(out, "{}", u16::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", u16::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::U32 => match data_format {
                DataFormat::Dec => write!(out, "{}", u32::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", u32::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::U64 => match data_format {
                DataFormat::Dec => write!(out, "{}", u64::read_le(buf.as_ptr().cast())).unwrap(),
                DataFormat::Hex => {
                    write!(out, "0x{:08X}", u64::read_le(buf.as_ptr().cast())).unwrap()
                }
                DataFormat::Bin => unreachable!(),
            },
            DataType::F32 => match data_format {
                DataFormat::Dec => {
                    write!(out, "{}", f32::from_bits(u32::read_be(buf.as_ptr().cast()))).unwrap()
                }
                _ => {}
            },
            DataType::F64 => match data_format {
                DataFormat::Dec => {
                    write!(out, "{}", f64::from_bits(u64::read_be(buf.as_ptr().cast()))).unwrap()
                }
                _ => {}
            },
        }
    }
}

fn format_binary(buf: &[u8], width: usize) -> String {
    let mut out_buf = String::new();
    let n = width / 8;

    for j in (0..=(n - 1)).rev() {
        for i in 0..8 {
            out_buf.push(if (buf[j] & (1 << (7 - i))) != 0 {
                '1'
            } else {
                '0'
            });
        }
        out_buf.push(' ');
    }

    out_buf
}
