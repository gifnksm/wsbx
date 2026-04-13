use std::{
    array,
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt as _, OsStringExt as _},
    str::EncodeUtf16,
};

#[derive(Debug)]
pub(crate) enum Xml {
    Element { name: OsString, content: Vec<Xml> },
    Text(OsString),
}

impl Xml {
    pub(crate) fn element<N, I, T>(name: N, content: I) -> Self
    where
        N: Into<OsString>,
        I: IntoIterator<Item = T>,
        T: Into<Xml>,
    {
        Xml::Element {
            name: name.into(),
            content: content.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn text<T>(text: T) -> Self
    where
        T: AsRef<OsStr>,
    {
        let text = text.as_ref();
        let wide = text
            .encode_wide()
            .flat_map(|wc| match wc {
                AMP => Escaped::Str("&amp;"),
                LT => Escaped::Str("&lt;"),
                GT => Escaped::Str("&gt;"),
                QUOT => Escaped::Str("&quot;"),
                APOS => Escaped::Str("&apos;"),
                _ => Escaped::Wide(wc),
            })
            .collect::<Vec<_>>();
        Xml::Text(OsString::from_wide(&wide))
    }

    pub(crate) fn to_os_string(&self) -> OsString {
        let mut output = OsString::new();
        self.to_os_string_impl(&mut output);
        output
    }

    pub(crate) fn to_pretty_os_string(&self) -> OsString {
        let mut output = OsString::new();
        self.to_os_string_pretty_impl(&mut output, 0);
        output
    }

    fn to_os_string_impl(&self, output: &mut OsString) {
        match self {
            Self::Element { name, content } => {
                open_tag(name, output);
                for child in content {
                    child.to_os_string_impl(output);
                }
                close_tag(name, output);
            }
            Self::Text(text) => {
                output.push(text);
            }
        }
    }

    fn to_os_string_pretty_impl(&self, output: &mut OsString, level: usize) {
        let indent = "  ".repeat(level);
        match self {
            Self::Element { name, content } => match content.as_slice() {
                [] => {
                    output.push(&indent);
                    open_tag(name, output);
                    close_tag(name, output);
                    output.push("\n");
                }
                [Xml::Text(text)] => {
                    output.push(&indent);
                    open_tag(name, output);
                    output.push(text);
                    close_tag(name, output);
                    output.push("\n");
                }
                content => {
                    output.push(&indent);
                    open_tag(name, output);
                    output.push("\n");
                    for child in content {
                        child.to_os_string_pretty_impl(output, level + 1);
                    }
                    output.push(&indent);
                    close_tag(name, output);
                    output.push("\n");
                }
            },
            Self::Text(text) => {
                output.push(&indent);
                output.push(text);
                output.push("\n");
            }
        }
    }
}

fn open_tag(name: &OsStr, output: &mut OsString) {
    output.push("<");
    output.push(name);
    output.push(">");
}

fn close_tag(name: &OsStr, output: &mut OsString) {
    output.push("</");
    output.push(name);
    output.push(">");
}

const fn utf16(c: char) -> u16 {
    let mut buf = [0; 2];
    let bytes = c.encode_utf16(&mut buf);
    assert!(bytes.len() == 1);
    bytes[0]
}

const AMP: u16 = utf16('&');
const LT: u16 = utf16('<');
const GT: u16 = utf16('>');
const QUOT: u16 = utf16('"');
const APOS: u16 = utf16('\'');

#[derive(Debug)]
enum Escaped {
    Str(&'static str),
    Wide(u16),
}

#[derive(Debug)]
enum EscapedIter {
    Str(EncodeUtf16<'static>),
    Wide(array::IntoIter<u16, 1>),
}

impl IntoIterator for Escaped {
    type IntoIter = EscapedIter;
    type Item = u16;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Str(s) => EscapedIter::Str(s.encode_utf16()),
            Self::Wide(w) => EscapedIter::Wide([w].into_iter()),
        }
    }
}

impl Iterator for EscapedIter {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Str(it) => it.next(),
            Self::Wide(it) => it.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::config::{Command, MappedFolder, SandboxConfig};

    use super::*;

    #[test]
    fn text_escapes_xml_special_characters() {
        let xml = Xml::text(r#"<tag attr="value">&'text'"#);

        assert_eq!(
            xml.to_os_string(),
            OsString::from("&lt;tag attr=&quot;value&quot;&gt;&amp;&apos;text&apos;")
        );
    }

    #[test]
    fn pretty_prints_nested_elements() {
        let xml = Xml::element(
            "Root",
            [
                Xml::element("Child", [Xml::text("value")]),
                Xml::element("Empty", std::iter::empty::<Xml>()),
            ],
        );

        assert_eq!(
            xml.to_pretty_os_string(),
            indoc! {r"
                <Root>
                  <Child>value</Child>
                  <Empty></Empty>
                </Root>
            "}
        );
    }

    #[test]
    fn config_serializes_to_expected_xml() {
        let config = SandboxConfig::new()
            .vgpu(false)
            .networking(true)
            .mapped_folder(
                MappedFolder::new(r"C:\host")
                    .sandbox_folder(r"C:\sandbox")
                    .read_only(true),
            )
            .logon_command(Command::new(r"C:\sandbox\setup.cmd"))
            .memory_in_mb(4096);

        assert_eq!(
            config.to_os_string(),
            concat!(
                r"<Configuration>",
                r"<vGPU>Disable</vGPU>",
                r"<Networking>Enable</Networking>",
                r"<MappedFolders>",
                r"<MappedFolder>",
                r"<HostFolder>C:\host</HostFolder>",
                r"<SandboxFolder>C:\sandbox</SandboxFolder>",
                r"<ReadOnly>true</ReadOnly>",
                r"</MappedFolder>",
                r"</MappedFolders>",
                r"<LogonCommand>",
                r"<Command>C:\sandbox\setup.cmd</Command>",
                r"</LogonCommand>",
                r"<MemoryInMB>4096</MemoryInMB>",
                r"</Configuration>"
            )
        );
    }
}
