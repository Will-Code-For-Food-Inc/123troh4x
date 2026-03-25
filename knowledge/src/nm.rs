//! Parser for `arm-none-eabi-nm` output.
//!
//! Handles two formats:
//!   `address kind name`             (plain nm)
//!   `address size kind name`        (nm --print-size)
//!
//! Lines where the address field is blank (undefined/external symbols) are
//! silently skipped, as are header lines and anything that doesn't parse.

use crate::db::Symbol;

/// Parse the stdout of `arm-none-eabi-nm [--print-size] [--numeric-sort]`.
/// Returns only defined symbols with a valid hex address.
pub fn parse_nm_output(output: &str) -> Vec<Symbol> {
    output.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Symbol> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.len() {
        3 => {
            // "address kind name" — plain nm, no size
            let address = u32::from_str_radix(parts[0], 16).ok()?;
            Some(Symbol::from_decomp(address, None, parts[1], parts[2]))
        }
        4 => {
            // "address size kind name" — nm --print-size
            let address = u32::from_str_radix(parts[0], 16).ok()?;
            let size    = u32::from_str_radix(parts[1], 16).ok();
            Some(Symbol::from_decomp(address, size, parts[2], parts[3]))
        }
        _ => None, // blank address (undefined), header lines, etc.
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_nm_three_columns() {
        let output = "08000000 T Reset_Handler\n08000100 T main\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[0].address, 0x0800_0000);
        assert_eq!(syms[0].kind, "T");
        assert_eq!(syms[0].name, "Reset_Handler");
        assert_eq!(syms[0].size, None);
    }

    #[test]
    fn nm_with_size_four_columns() {
        let output = "08000000 00000004 T Reset_Handler\n08000004 000000fc T main\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[0].size, Some(4));
        assert_eq!(syms[1].size, Some(0xfc));
    }

    #[test]
    fn undefined_symbols_skipped() {
        // Undefined symbols have blank address fields — split_whitespace gives 2 tokens
        let output = "         U _estack\n08000000 T main\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "main");
    }

    #[test]
    fn empty_lines_skipped() {
        let output = "\n\n08000000 T main\n\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 1);
    }

    #[test]
    fn header_lines_skipped() {
        // nm sometimes emits a filename header
        let output = "pokemon.elf:\n08000000 T main\n";
        let syms = parse_nm_output(output);
        // "pokemon.elf:" splits to 1 token → falls through to _ => None
        assert_eq!(syms.len(), 1);
    }

    #[test]
    fn local_and_global_symbols() {
        // Lowercase kind = local, uppercase = global
        let output = "08000000 T GlobalFunc\n08000010 t local_helper\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[0].kind, "T");
        assert_eq!(syms[1].kind, "t");
    }

    #[test]
    fn data_and_bss_symbols() {
        let output = "20000000 00000004 D gCounter\n20000004 00000100 B gBuffer\n";
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 2);
        assert_eq!(syms[0].kind, "D");
        assert_eq!(syms[1].kind, "B");
    }

    #[test]
    fn mixed_output_roundtrip() {
        let output = concat!(
            "         U external_dep\n",
            "08000000 00000004 T Reset_Handler\n",
            "08000004 000000f8 T main\n",
            "080000fc 0000000c t init_bss\n",
            "20000000 00000010 D gConfig\n",
            "20000010 00000200 B gStack\n",
        );
        let syms = parse_nm_output(output);
        assert_eq!(syms.len(), 5); // external_dep skipped
        assert_eq!(syms[0].address, 0x0800_0000);
        assert_eq!(syms[4].address, 0x2000_0010);
    }

    #[test]
    fn empty_input() {
        assert!(parse_nm_output("").is_empty());
    }
}
