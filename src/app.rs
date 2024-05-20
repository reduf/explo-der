use imgui::*;
use crate::error::{Error, Result};
use crate::{hex, oid, oid_names, der::{self, Tag}};

pub struct App {
    signal_stop: bool,
    next_id: usize,
    tree: Option<Tree>,
}

pub struct Tree {
    pub expanded: bool,
    pub subtrees: Vec<Tree>,
}

impl Tree {
    pub fn new() -> Self {
        return Self { expanded: true, subtrees: Vec::new() };
    }
}

impl App {
    pub fn new() -> Self {
        return Self {
            signal_stop: false,
            next_id: 0,
            tree: Some(Tree::new()),
        };
    }

    pub fn should_quit(&self) -> bool {
        return self.signal_stop;
    }

    fn push_next_id<'a>(&mut self, ui: &'a Ui) -> IdStackToken<'a> {
        let token = ui.push_id_usize(self.next_id);
        self.next_id += 1;
        return token;
    }

    pub fn update(&mut self, ui: &Ui) {
        let window_size = ui.io().display_size;

        // self.settings.draw_settings(ui);

        let window = ui
            .window("Explo-DER##main")
            .position([0.0, 0.0], Condition::FirstUseEver)
            .size(window_size, Condition::Always)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .bring_to_front_on_focus(false)
            .menu_bar(true);

        window.build(|| {
            if let Some(_) = ui.begin_menu_bar() {
                // self.draw_menu(ui);
            }

            self.draw_main_content(ui);
        });
    }

    pub fn draw_boolean(&mut self, ui: &Ui, content: &[u8]) {
        match content {
            [0] => ui.text("false"),
            [255] => ui.text("false"),
            _ => ui.text("Invalid boolean"),
        }
    }

    fn draw_tree_helper<'a>(ui: &'a Ui, label: &str) -> Option<TreeNodeToken<'a>> {
        if let Some(token) = ui.tree_node_config(label).opened(true, Condition::Once).push() {
            if ui.is_item_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
                ui.open_popup("toto");
            }

            if let Some(_) = ui.begin_popup("toto") {
                if ui.menu_item_config("Copy Bytes").build() {
                    // ui.set_clipboard_text(full_path.as_ref());
                }

                if ui.menu_item_config("View bytes").build() {
                }
            }

            return Some(token);
        } else {
            return None;
        }
    }

    pub fn draw_sequence(&mut self, ui: &Ui, tree: &mut Tree, content: &[u8]) -> Result<()> {
        let label = format!("{:?} - len: {}", Tag::Sequence, content.len());
        if let Some(_) = Self::draw_tree_helper(ui, &label) {
            self.draw_tree(ui, tree, content)?;
        }
        return Ok(());
    }

    pub fn draw_set(&mut self, ui: &Ui, tree: &mut Tree, content: &[u8]) -> Result<()> {
        let label = format!("{:?} - len: {}", Tag::Set, content.len());
        if let Some(_) = Self::draw_tree_helper(ui, &label) {
            self.draw_tree(ui, tree, content)?;
        }
        return Ok(());
    }

    pub fn draw_oid(&mut self, ui: &Ui, content: &[u8]) {
        if let Ok(oid) = oid::stringify(&content) {
            if let Some(oid_name) = oid_names::find(&oid) {
                ui.text(&format!("{:?} - {} {}", Tag::OID, oid, oid_name));
            } else {
                ui.text(&format!("{:?} - {}", Tag::OID, oid));
            }

            if let Some(_) = ui.begin_popup("copy-oid") {
                if ui.menu_item_config("Copy OID as dot notation").build() {
                    ui.set_clipboard_text(oid);
                }
                if ui.menu_item_config("Copy OID as bytes").build() {
                    ui.set_clipboard_text(format!("{:02X?}", content));
                }
            }
        } else {
            ui.text(&format!("{:?} - len: {}", Tag::OID, content.len()));
        }

        if ui.is_item_hovered() && ui.is_mouse_clicked(MouseButton::Right) {
            ui.open_popup("copy-oid");
        }
    }

    pub fn draw_octet_string(&mut self, ui: &Ui, tree: &mut Tree, content: &[u8]) -> Result<()> {
        let label = format!("{:?} - {}", Tag::OctetString, hex::hexlify(content));
        if let Some(_) = Self::draw_tree_helper(ui, &label) {
            self.draw_tree(ui, tree, content)?;
        }
        return Ok(());
    }

    pub fn draw_context_specific_constructed(&mut self, ui: &Ui, tree: &mut Tree, tag: Tag, content: &[u8]) -> Result<()> {
        let label = format!("{:?} - len: {}", tag, content.len());
        if let Some(_) = Self::draw_tree_helper(ui, &label) {
            self.draw_tree(ui, tree, content)?;
        }
        return Ok(());
    }

    pub fn draw_printable_string(&mut self, ui: &Ui, content: &[u8]) -> Result<()> {
        let value = std::str::from_utf8(content).map_err(|_| Error("Invalid UTF8 string"))?;
        ui.text(format!("{:?} - '{}'", Tag::Utf8String, value));
        return Ok(());
    }

    pub fn draw_bmp_string(&mut self, ui: &Ui, content: &[u8]) -> Result<()> {
        let codepoints = content
            .chunks_exact(2)
            .map(|bytes| ((bytes[0] as u16) << 8) | bytes[1] as u16)
            .collect::<Vec<u16>>();
        let value = String::from_utf16(codepoints.as_slice()).unwrap();
        ui.text(format!("{:?} - '{}'", Tag::BMPString, value));
        return Ok(());
    }

    pub fn draw_utf8_string(&mut self, ui: &Ui, content: &[u8]) -> Result<()> {
        let mut builder = String::with_capacity(content.len());
        for &byte in content {
            match byte {
                b'A'..=b'Z' => builder.push(byte as char),
                b'a'..=b'z' => builder.push(byte as char),
                b'0'..=b'9' => builder.push(byte as char),
                b' ' | b'\'' | b'(' | b')' | b'+' | b',' | b'-' | b'.' | b'/' | b':' | b'=' | b'?' => builder.push(byte as char),
                _ => return Err(Error("Invalid PrintableString")),
            }
        }

        ui.text(format!("{:?} - '{}'", Tag::PrintableString, builder));
        return Ok(());
    }

    pub fn draw_time(&mut self, ui: &Ui, tag: Tag, content: &[u8]) -> Result<()> {
        use chrono::{offset::LocalResult, TimeZone, Utc};

        fn read_digit(inner: &mut der::Reader) -> Result<u32> {
            let byte = inner.read_byte()?;
            return match byte {
                b'0'..=b'9' => Ok(u32::from(byte - b'0')),
                _ => Err(Error("Invalid digit found")),
            };
        }

        fn read_two_digits(inner: &mut der::Reader, min: u32, max: u32) -> Result<u32> {
            let hi = read_digit(inner)?;
            let lo = read_digit(inner)?;
            let value = (hi * 10) + lo;
            if value < min || value > max {
                return Err(Error("Digit outside expected range"));
            }
            return Ok(value);
        }

        fn get_days_in_month(year: i32, month: u32) -> Result<u32> {
            let next_month = if month == 12 {
                chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
            }.ok_or(Error("Can't initialize a 'chrono::NativeDate'"))?;

            let duration_since = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or(Error("Invalid year or month"))?;
            let number_of_days = next_month
                .signed_duration_since(duration_since)
                .num_days();
            assert!(number_of_days <= (u32::MAX as i64));
            return Ok(number_of_days as u32);
        }

        let mut reader = der::Reader::new(content);

        let (year_hi, year_lo) = if tag == Tag::UTCTime {
            let lo = read_two_digits(&mut reader, 0, 99)?;
            let hi = if lo >= 50 { 19 } else { 20 };
            (hi, lo)
        } else {
            let hi = read_two_digits(&mut reader, 0, 99)?;
            let lo = read_two_digits(&mut reader, 0, 99)?;
            (hi, lo)
        };

        let year = {
            let y = (year_hi * 100) + year_lo;
            assert!(y <= (i32::MAX as u32));
            y as i32
        };

        let month = read_two_digits(&mut reader, 1, 12)?;
        let days_in_month = get_days_in_month(year, month)?;
        let day_of_month = read_two_digits(&mut reader, 1, days_in_month)?;
        let hours = read_two_digits(&mut reader, 0, 23)?;
        let minutes = read_two_digits(&mut reader, 0, 59)?;
        let seconds = read_two_digits(&mut reader, 0, 59)?;

        if reader.read_byte()? != b'Z' {
            return Err(Error("Invalid or unsupported timezone"));
        }

        let time_as_utc = match Utc.with_ymd_and_hms(year, month, day_of_month, hours, minutes, seconds) {
            LocalResult::None => Err(Error("Can't initialize a Utc time")),
            LocalResult::Single(dt) => Ok(dt),
            LocalResult::Ambiguous(_dt1, _dt2) => {
                /*
                log::error!(
                    "Ambiguous time created with year={}, month={}, day_of_month={}, hours={}, minutes={}, seconds={}, result in dt1={}, dt2={}",
                    year,
                    month,
                    day_of_month,
                    hours,
                    minutes,
                    seconds,
                    dt1,
                    dt2,
                );
                */
                return Err(Error("Ambiguous time"));
            }
        }?;

        ui.text(format!("{:?} - {}", tag, time_as_utc));
        return Ok(());
    }

    pub fn draw_integer(&mut self, ui: &Ui, content: &[u8]) {
        ui.text(format!("{:?} - {}", Tag::Integer, content.len()));
    }

    pub fn draw_tree(&mut self, ui: &Ui, tree: &mut Tree, content: &[u8]) -> Result<()> {
        let mut idx = 0;
        let mut reader = der::Reader::new(content);

        while !reader.at_end() {
            if tree.subtrees.len() <= idx {
                tree.subtrees.push(Tree::new());
            }

            let tree = &mut tree.subtrees[idx];

            let remaining_bytes = reader.len();
            match der::read_tag_and_get_value(&mut reader) {
                Ok((Tag::Boolean, sub_content)) => self.draw_boolean(ui, sub_content),
                Ok((Tag::Integer, sub_content)) => self.draw_integer(ui, sub_content),
                Ok((Tag::OctetString, sub_content)) => self.draw_octet_string(ui, tree, sub_content)?,
                Ok((Tag::OID, sub_content)) => self.draw_oid(ui, sub_content),
                Ok((Tag::Utf8String, sub_content)) => self.draw_utf8_string(ui, sub_content)?,
                Ok((Tag::Sequence, sub_content)) => self.draw_sequence(ui, tree, sub_content)?,
                Ok((Tag::Set, sub_content)) => self.draw_set(ui, tree, sub_content)?,
                Ok((Tag::PrintableString, sub_content)) => self.draw_printable_string(ui, sub_content)?,
                Ok((Tag::BMPString, sub_content)) => self.draw_bmp_string(ui, sub_content)?,
                Ok((Tag::UTCTime, sub_content)) => self.draw_time(ui, Tag::UTCTime, sub_content)?,
                Ok((Tag::GeneralizedTime, sub_content)) => self.draw_time(ui, Tag::GeneralizedTime, sub_content)?,
                Ok((tag, sub_content)) if tag == Tag::ContextSpecificConstructed0 => self.draw_context_specific_constructed(ui, tree, tag, sub_content)?,
                Ok((tag, sub_content)) if tag == Tag::ContextSpecificConstructed1 => self.draw_context_specific_constructed(ui, tree, tag, sub_content)?,
                Ok((tag, sub_content)) if tag == Tag::ContextSpecificConstructed2 => self.draw_context_specific_constructed(ui, tree, tag, sub_content)?,
                Ok((tag, sub_content)) if tag == Tag::ContextSpecificConstructed3 => self.draw_context_specific_constructed(ui, tree, tag, sub_content)?,
                Ok((tag, sub_content)) => ui.text(&format!("{:?} - len: {}", tag, sub_content.len())),
                Err(err) => {
                    ui.text(&format!("{} bytes remaining, err: {}", remaining_bytes, err));
                    break;
                },
            }

            idx += 1;
        }

        return Ok(());
    }

    pub fn draw_main_content(&mut self, ui: &Ui) {
        let bytes = include_bytes!("../tests/no-password.pfx");

        let mut tree = self.tree.take().unwrap();
        self.draw_tree(ui, &mut tree, bytes.as_ref()).unwrap();
        self.tree = Some(tree);
    }
}
