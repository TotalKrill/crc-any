#[cfg(feature = "default")]
use alloc::fmt::{self, Formatter, Display, Debug};

/// This struct can help you compute a CRC-8 (or CRC-x where **x** is under `8`) value.
pub struct CRCu8 {
    by_table: bool,
    poly: u8,
    lookup_table: [u8; 256],
    sum: u8,
    #[cfg(feature = "default")]
    pub(crate) bits: u8,
    high_bit: u8,
    mask: u8,
    initial: u8,
    final_xor: u8,
    reflect: bool,
}

#[cfg(feature = "default")]
impl Debug for CRCu8 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.by_table {
            impl_debug_for_struct!(CRCu64, f, self, let .lookup_table = self.lookup_table.as_ref(), (.sum, "0x{:02X}", self.sum), .bits, (.initial, "0x{:02X}", self.initial), (.final_xor, "0x{:02X}", self.final_xor), .reflect);
        } else {
            impl_debug_for_struct!(CRCu64, f, self, (.poly, "0x{:02X}", self.poly), (.sum, "0x{:02X}", self.sum), .bits, (.initial, "0x{:02X}", self.initial), (.final_xor, "0x{:02X}", self.final_xor), .reflect);
        }
    }
}

#[cfg(feature = "default")]
impl Display for CRCu8 {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_fmt(format_args!("0x{:01$X}", self.get_crc(), ((self.bits as f64 + 3f64) / 4f64) as usize))
    }
}

impl CRCu8 {
    /// Create a `CRCu8` instance by providing the length of bits, expression, reflection, an initial value and a final xor value.
    pub fn create_crc(poly: u8, bits: u8, initial: u8, final_xor: u8, reflect: bool) -> CRCu8 {
        debug_assert!(bits <= 8 && bits > 0);

        if bits % 8 == 0 {
            let lookup_table = if reflect {
                Self::crc_reflect_table(poly)
            } else {
                Self::crc_table(poly)
            };

            Self::create_crc_with_exists_lookup_table(lookup_table, bits, initial, final_xor, reflect)
        } else {
            Self::create(false, [0u8; 256], poly, bits, initial, final_xor, reflect)
        }
    }

    #[inline]
    pub(crate) fn create_crc_with_exists_lookup_table(lookup_table: [u8; 256], bits: u8, initial: u8, final_xor: u8, reflect: bool) -> CRCu8 {
        debug_assert!(bits % 8 == 0);

        Self::create(true, lookup_table, 0, bits, initial, final_xor, reflect)
    }

    #[inline]
    fn create(by_table: bool, lookup_table: [u8; 256], mut poly: u8, bits: u8, initial: u8, final_xor: u8, reflect: bool) -> CRCu8 {
        let high_bit = 1 << (bits - 1);
        let mask = ((high_bit - 1) << 1) | 1;

        let sum = if reflect {
            Self::reflect_function(high_bit, initial)
        } else {
            initial
        };

        if !by_table && reflect {
            poly = Self::reflect_function(high_bit, poly);
        }

        CRCu8 {
            by_table,
            poly,
            lookup_table,
            sum,
            #[cfg(feature = "default")]
            bits,
            high_bit,
            mask,
            initial,
            final_xor,
            reflect,
        }
    }

    #[inline]
    pub(crate) fn reflect_function(high_bit: u8, n: u8) -> u8 {
        let mut i = high_bit;
        let mut j = 1;
        let mut out = 0;

        while i != 0 {
            if n & i != 0 {
                out |= j;
            }

            j <<= 1;
            i >>= 1;
        }

        out
    }

    #[inline]
    fn reflect_method(&self, n: u8) -> u8 {
        Self::reflect_function(self.high_bit, n)
    }

    /// Digest some data.
    pub fn digest<T: ?Sized + AsRef<[u8]>>(&mut self, data: &T) {
        if self.by_table {
            for &n in data.as_ref() {
                let index = (self.sum ^ n) as usize;
                self.sum = self.lookup_table[index];
            }
        } else {
            if self.reflect {
                for &n in data.as_ref() {
                    let n = super::crc_u8::CRCu8::reflect_function(0x80, n);

                    let mut i = 0x80;

                    while i != 0 {
                        let mut bit = self.sum & self.high_bit;

                        self.sum <<= 1;

                        if n & i != 0 {
                            bit ^= self.high_bit;
                        }


                        if bit != 0 {
                            self.sum ^= self.poly;
                        }

                        i >>= 1;
                    }
                }
            } else {
                for &n in data.as_ref() {
                    let mut i = 0x80;

                    while i != 0 {
                        let mut bit = self.sum & self.high_bit;

                        self.sum <<= 1;

                        if n & i != 0 {
                            bit ^= self.high_bit;
                        }


                        if bit != 0 {
                            self.sum ^= self.poly;
                        }

                        i >>= 1;
                    }
                }
            }
        }
    }

    /// Reset the sum.
    pub fn reset(&mut self) {
        self.sum = self.initial;
    }

    /// Get the current CRC value (it always returns a `u8` value). You can continue calling `digest` method even after getting a CRC value.
    pub fn get_crc(&self) -> u8 {
        if self.by_table {
            (self.sum ^ self.final_xor) & self.mask
        } else {
            if self.reflect {
                (self.reflect_method(self.sum) ^ self.final_xor) & self.mask
            } else {
                (self.sum ^ self.final_xor) & self.mask
            }
        }
    }

    fn crc_reflect_table(poly_rev: u8) -> [u8; 256] {
        let mut lookup_table = [0u8; 256];

        for i in 0..=255 {
            let mut v = i as u8;

            for _ in 0..8u8 {
                if v & 1 != 0 {
                    v >>= 1;
                    v ^= poly_rev;
                } else {
                    v >>= 1;
                }
            }

            lookup_table[i] = v;
        }

        lookup_table
    }

    fn crc_table(poly: u8) -> [u8; 256] {
        let mut lookup_table = [0u8; 256];

        for i in 0..=255 {
            let mut v = i as u8;

            for _ in 0..8 {
                if v & 0x80 == 0 {
                    v <<= 1;
                } else {
                    v <<= 1;
                    v ^= poly;
                }
            }

            lookup_table[i] = v & 0xFF;
        }

        lookup_table
    }
}

const NO_REF_8_07: [u8; 256] = [0u8, 7u8, 14u8, 9u8, 28u8, 27u8, 18u8, 21u8, 56u8, 63u8, 54u8, 49u8, 36u8, 35u8, 42u8, 45u8, 112u8, 119u8, 126u8, 121u8, 108u8, 107u8, 98u8, 101u8, 72u8, 79u8, 70u8, 65u8, 84u8, 83u8, 90u8, 93u8, 224u8, 231u8, 238u8, 233u8, 252u8, 251u8, 242u8, 245u8, 216u8, 223u8, 214u8, 209u8, 196u8, 195u8, 202u8, 205u8, 144u8, 151u8, 158u8, 153u8, 140u8, 139u8, 130u8, 133u8, 168u8, 175u8, 166u8, 161u8, 180u8, 179u8, 186u8, 189u8, 199u8, 192u8, 201u8, 206u8, 219u8, 220u8, 213u8, 210u8, 255u8, 248u8, 241u8, 246u8, 227u8, 228u8, 237u8, 234u8, 183u8, 176u8, 185u8, 190u8, 171u8, 172u8, 165u8, 162u8, 143u8, 136u8, 129u8, 134u8, 147u8, 148u8, 157u8, 154u8, 39u8, 32u8, 41u8, 46u8, 59u8, 60u8, 53u8, 50u8, 31u8, 24u8, 17u8, 22u8, 3u8, 4u8, 13u8, 10u8, 87u8, 80u8, 89u8, 94u8, 75u8, 76u8, 69u8, 66u8, 111u8, 104u8, 97u8, 102u8, 115u8, 116u8, 125u8, 122u8, 137u8, 142u8, 135u8, 128u8, 149u8, 146u8, 155u8, 156u8, 177u8, 182u8, 191u8, 184u8, 173u8, 170u8, 163u8, 164u8, 249u8, 254u8, 247u8, 240u8, 229u8, 226u8, 235u8, 236u8, 193u8, 198u8, 207u8, 200u8, 221u8, 218u8, 211u8, 212u8, 105u8, 110u8, 103u8, 96u8, 117u8, 114u8, 123u8, 124u8, 81u8, 86u8, 95u8, 88u8, 77u8, 74u8, 67u8, 68u8, 25u8, 30u8, 23u8, 16u8, 5u8, 2u8, 11u8, 12u8, 33u8, 38u8, 47u8, 40u8, 61u8, 58u8, 51u8, 52u8, 78u8, 73u8, 64u8, 71u8, 82u8, 85u8, 92u8, 91u8, 118u8, 113u8, 120u8, 127u8, 106u8, 109u8, 100u8, 99u8, 62u8, 57u8, 48u8, 55u8, 34u8, 37u8, 44u8, 43u8, 6u8, 1u8, 8u8, 15u8, 26u8, 29u8, 20u8, 19u8, 174u8, 169u8, 160u8, 167u8, 178u8, 181u8, 188u8, 187u8, 150u8, 145u8, 152u8, 159u8, 138u8, 141u8, 132u8, 131u8, 222u8, 217u8, 208u8, 215u8, 194u8, 197u8, 204u8, 203u8, 230u8, 225u8, 232u8, 239u8, 250u8, 253u8, 244u8, 243u8];
const NO_REF_8_1D: [u8; 256] = [0u8, 29u8, 58u8, 39u8, 116u8, 105u8, 78u8, 83u8, 232u8, 245u8, 210u8, 207u8, 156u8, 129u8, 166u8, 187u8, 205u8, 208u8, 247u8, 234u8, 185u8, 164u8, 131u8, 158u8, 37u8, 56u8, 31u8, 2u8, 81u8, 76u8, 107u8, 118u8, 135u8, 154u8, 189u8, 160u8, 243u8, 238u8, 201u8, 212u8, 111u8, 114u8, 85u8, 72u8, 27u8, 6u8, 33u8, 60u8, 74u8, 87u8, 112u8, 109u8, 62u8, 35u8, 4u8, 25u8, 162u8, 191u8, 152u8, 133u8, 214u8, 203u8, 236u8, 241u8, 19u8, 14u8, 41u8, 52u8, 103u8, 122u8, 93u8, 64u8, 251u8, 230u8, 193u8, 220u8, 143u8, 146u8, 181u8, 168u8, 222u8, 195u8, 228u8, 249u8, 170u8, 183u8, 144u8, 141u8, 54u8, 43u8, 12u8, 17u8, 66u8, 95u8, 120u8, 101u8, 148u8, 137u8, 174u8, 179u8, 224u8, 253u8, 218u8, 199u8, 124u8, 97u8, 70u8, 91u8, 8u8, 21u8, 50u8, 47u8, 89u8, 68u8, 99u8, 126u8, 45u8, 48u8, 23u8, 10u8, 177u8, 172u8, 139u8, 150u8, 197u8, 216u8, 255u8, 226u8, 38u8, 59u8, 28u8, 1u8, 82u8, 79u8, 104u8, 117u8, 206u8, 211u8, 244u8, 233u8, 186u8, 167u8, 128u8, 157u8, 235u8, 246u8, 209u8, 204u8, 159u8, 130u8, 165u8, 184u8, 3u8, 30u8, 57u8, 36u8, 119u8, 106u8, 77u8, 80u8, 161u8, 188u8, 155u8, 134u8, 213u8, 200u8, 239u8, 242u8, 73u8, 84u8, 115u8, 110u8, 61u8, 32u8, 7u8, 26u8, 108u8, 113u8, 86u8, 75u8, 24u8, 5u8, 34u8, 63u8, 132u8, 153u8, 190u8, 163u8, 240u8, 237u8, 202u8, 215u8, 53u8, 40u8, 15u8, 18u8, 65u8, 92u8, 123u8, 102u8, 221u8, 192u8, 231u8, 250u8, 169u8, 180u8, 147u8, 142u8, 248u8, 229u8, 194u8, 223u8, 140u8, 145u8, 182u8, 171u8, 16u8, 13u8, 42u8, 55u8, 100u8, 121u8, 94u8, 67u8, 178u8, 175u8, 136u8, 149u8, 198u8, 219u8, 252u8, 225u8, 90u8, 71u8, 96u8, 125u8, 46u8, 51u8, 20u8, 9u8, 127u8, 98u8, 69u8, 88u8, 11u8, 22u8, 49u8, 44u8, 151u8, 138u8, 173u8, 176u8, 227u8, 254u8, 217u8, 196u8];
const NO_REF_8_D5: [u8; 256] = [0u8, 213u8, 127u8, 170u8, 254u8, 43u8, 129u8, 84u8, 41u8, 252u8, 86u8, 131u8, 215u8, 2u8, 168u8, 125u8, 82u8, 135u8, 45u8, 248u8, 172u8, 121u8, 211u8, 6u8, 123u8, 174u8, 4u8, 209u8, 133u8, 80u8, 250u8, 47u8, 164u8, 113u8, 219u8, 14u8, 90u8, 143u8, 37u8, 240u8, 141u8, 88u8, 242u8, 39u8, 115u8, 166u8, 12u8, 217u8, 246u8, 35u8, 137u8, 92u8, 8u8, 221u8, 119u8, 162u8, 223u8, 10u8, 160u8, 117u8, 33u8, 244u8, 94u8, 139u8, 157u8, 72u8, 226u8, 55u8, 99u8, 182u8, 28u8, 201u8, 180u8, 97u8, 203u8, 30u8, 74u8, 159u8, 53u8, 224u8, 207u8, 26u8, 176u8, 101u8, 49u8, 228u8, 78u8, 155u8, 230u8, 51u8, 153u8, 76u8, 24u8, 205u8, 103u8, 178u8, 57u8, 236u8, 70u8, 147u8, 199u8, 18u8, 184u8, 109u8, 16u8, 197u8, 111u8, 186u8, 238u8, 59u8, 145u8, 68u8, 107u8, 190u8, 20u8, 193u8, 149u8, 64u8, 234u8, 63u8, 66u8, 151u8, 61u8, 232u8, 188u8, 105u8, 195u8, 22u8, 239u8, 58u8, 144u8, 69u8, 17u8, 196u8, 110u8, 187u8, 198u8, 19u8, 185u8, 108u8, 56u8, 237u8, 71u8, 146u8, 189u8, 104u8, 194u8, 23u8, 67u8, 150u8, 60u8, 233u8, 148u8, 65u8, 235u8, 62u8, 106u8, 191u8, 21u8, 192u8, 75u8, 158u8, 52u8, 225u8, 181u8, 96u8, 202u8, 31u8, 98u8, 183u8, 29u8, 200u8, 156u8, 73u8, 227u8, 54u8, 25u8, 204u8, 102u8, 179u8, 231u8, 50u8, 152u8, 77u8, 48u8, 229u8, 79u8, 154u8, 206u8, 27u8, 177u8, 100u8, 114u8, 167u8, 13u8, 216u8, 140u8, 89u8, 243u8, 38u8, 91u8, 142u8, 36u8, 241u8, 165u8, 112u8, 218u8, 15u8, 32u8, 245u8, 95u8, 138u8, 222u8, 11u8, 161u8, 116u8, 9u8, 220u8, 118u8, 163u8, 247u8, 34u8, 136u8, 93u8, 214u8, 3u8, 169u8, 124u8, 40u8, 253u8, 87u8, 130u8, 255u8, 42u8, 128u8, 85u8, 1u8, 212u8, 126u8, 171u8, 132u8, 81u8, 251u8, 46u8, 122u8, 175u8, 5u8, 208u8, 173u8, 120u8, 210u8, 7u8, 83u8, 134u8, 44u8, 249u8];
const NO_REF_8_9B: [u8; 256] = [0u8, 155u8, 173u8, 54u8, 193u8, 90u8, 108u8, 247u8, 25u8, 130u8, 180u8, 47u8, 216u8, 67u8, 117u8, 238u8, 50u8, 169u8, 159u8, 4u8, 243u8, 104u8, 94u8, 197u8, 43u8, 176u8, 134u8, 29u8, 234u8, 113u8, 71u8, 220u8, 100u8, 255u8, 201u8, 82u8, 165u8, 62u8, 8u8, 147u8, 125u8, 230u8, 208u8, 75u8, 188u8, 39u8, 17u8, 138u8, 86u8, 205u8, 251u8, 96u8, 151u8, 12u8, 58u8, 161u8, 79u8, 212u8, 226u8, 121u8, 142u8, 21u8, 35u8, 184u8, 200u8, 83u8, 101u8, 254u8, 9u8, 146u8, 164u8, 63u8, 209u8, 74u8, 124u8, 231u8, 16u8, 139u8, 189u8, 38u8, 250u8, 97u8, 87u8, 204u8, 59u8, 160u8, 150u8, 13u8, 227u8, 120u8, 78u8, 213u8, 34u8, 185u8, 143u8, 20u8, 172u8, 55u8, 1u8, 154u8, 109u8, 246u8, 192u8, 91u8, 181u8, 46u8, 24u8, 131u8, 116u8, 239u8, 217u8, 66u8, 158u8, 5u8, 51u8, 168u8, 95u8, 196u8, 242u8, 105u8, 135u8, 28u8, 42u8, 177u8, 70u8, 221u8, 235u8, 112u8, 11u8, 144u8, 166u8, 61u8, 202u8, 81u8, 103u8, 252u8, 18u8, 137u8, 191u8, 36u8, 211u8, 72u8, 126u8, 229u8, 57u8, 162u8, 148u8, 15u8, 248u8, 99u8, 85u8, 206u8, 32u8, 187u8, 141u8, 22u8, 225u8, 122u8, 76u8, 215u8, 111u8, 244u8, 194u8, 89u8, 174u8, 53u8, 3u8, 152u8, 118u8, 237u8, 219u8, 64u8, 183u8, 44u8, 26u8, 129u8, 93u8, 198u8, 240u8, 107u8, 156u8, 7u8, 49u8, 170u8, 68u8, 223u8, 233u8, 114u8, 133u8, 30u8, 40u8, 179u8, 195u8, 88u8, 110u8, 245u8, 2u8, 153u8, 175u8, 52u8, 218u8, 65u8, 119u8, 236u8, 27u8, 128u8, 182u8, 45u8, 241u8, 106u8, 92u8, 199u8, 48u8, 171u8, 157u8, 6u8, 232u8, 115u8, 69u8, 222u8, 41u8, 178u8, 132u8, 31u8, 167u8, 60u8, 10u8, 145u8, 102u8, 253u8, 203u8, 80u8, 190u8, 37u8, 19u8, 136u8, 127u8, 228u8, 210u8, 73u8, 149u8, 14u8, 56u8, 163u8, 84u8, 207u8, 249u8, 98u8, 140u8, 23u8, 33u8, 186u8, 77u8, 214u8, 224u8, 123u8];

const REF_8_8C: [u8; 256] = [0u8, 94u8, 188u8, 226u8, 97u8, 63u8, 221u8, 131u8, 194u8, 156u8, 126u8, 32u8, 163u8, 253u8, 31u8, 65u8, 157u8, 195u8, 33u8, 127u8, 252u8, 162u8, 64u8, 30u8, 95u8, 1u8, 227u8, 189u8, 62u8, 96u8, 130u8, 220u8, 35u8, 125u8, 159u8, 193u8, 66u8, 28u8, 254u8, 160u8, 225u8, 191u8, 93u8, 3u8, 128u8, 222u8, 60u8, 98u8, 190u8, 224u8, 2u8, 92u8, 223u8, 129u8, 99u8, 61u8, 124u8, 34u8, 192u8, 158u8, 29u8, 67u8, 161u8, 255u8, 70u8, 24u8, 250u8, 164u8, 39u8, 121u8, 155u8, 197u8, 132u8, 218u8, 56u8, 102u8, 229u8, 187u8, 89u8, 7u8, 219u8, 133u8, 103u8, 57u8, 186u8, 228u8, 6u8, 88u8, 25u8, 71u8, 165u8, 251u8, 120u8, 38u8, 196u8, 154u8, 101u8, 59u8, 217u8, 135u8, 4u8, 90u8, 184u8, 230u8, 167u8, 249u8, 27u8, 69u8, 198u8, 152u8, 122u8, 36u8, 248u8, 166u8, 68u8, 26u8, 153u8, 199u8, 37u8, 123u8, 58u8, 100u8, 134u8, 216u8, 91u8, 5u8, 231u8, 185u8, 140u8, 210u8, 48u8, 110u8, 237u8, 179u8, 81u8, 15u8, 78u8, 16u8, 242u8, 172u8, 47u8, 113u8, 147u8, 205u8, 17u8, 79u8, 173u8, 243u8, 112u8, 46u8, 204u8, 146u8, 211u8, 141u8, 111u8, 49u8, 178u8, 236u8, 14u8, 80u8, 175u8, 241u8, 19u8, 77u8, 206u8, 144u8, 114u8, 44u8, 109u8, 51u8, 209u8, 143u8, 12u8, 82u8, 176u8, 238u8, 50u8, 108u8, 142u8, 208u8, 83u8, 13u8, 239u8, 177u8, 240u8, 174u8, 76u8, 18u8, 145u8, 207u8, 45u8, 115u8, 202u8, 148u8, 118u8, 40u8, 171u8, 245u8, 23u8, 73u8, 8u8, 86u8, 180u8, 234u8, 105u8, 55u8, 213u8, 139u8, 87u8, 9u8, 235u8, 181u8, 54u8, 104u8, 138u8, 212u8, 149u8, 203u8, 41u8, 119u8, 244u8, 170u8, 72u8, 22u8, 233u8, 183u8, 85u8, 11u8, 136u8, 214u8, 52u8, 106u8, 43u8, 117u8, 151u8, 201u8, 74u8, 20u8, 246u8, 168u8, 116u8, 42u8, 200u8, 150u8, 21u8, 75u8, 169u8, 247u8, 182u8, 232u8, 10u8, 84u8, 215u8, 137u8, 107u8, 53u8];
const REF_8_9C: [u8; 256] = [0u8, 114u8, 228u8, 150u8, 241u8, 131u8, 21u8, 103u8, 219u8, 169u8, 63u8, 77u8, 42u8, 88u8, 206u8, 188u8, 143u8, 253u8, 107u8, 25u8, 126u8, 12u8, 154u8, 232u8, 84u8, 38u8, 176u8, 194u8, 165u8, 215u8, 65u8, 51u8, 39u8, 85u8, 195u8, 177u8, 214u8, 164u8, 50u8, 64u8, 252u8, 142u8, 24u8, 106u8, 13u8, 127u8, 233u8, 155u8, 168u8, 218u8, 76u8, 62u8, 89u8, 43u8, 189u8, 207u8, 115u8, 1u8, 151u8, 229u8, 130u8, 240u8, 102u8, 20u8, 78u8, 60u8, 170u8, 216u8, 191u8, 205u8, 91u8, 41u8, 149u8, 231u8, 113u8, 3u8, 100u8, 22u8, 128u8, 242u8, 193u8, 179u8, 37u8, 87u8, 48u8, 66u8, 212u8, 166u8, 26u8, 104u8, 254u8, 140u8, 235u8, 153u8, 15u8, 125u8, 105u8, 27u8, 141u8, 255u8, 152u8, 234u8, 124u8, 14u8, 178u8, 192u8, 86u8, 36u8, 67u8, 49u8, 167u8, 213u8, 230u8, 148u8, 2u8, 112u8, 23u8, 101u8, 243u8, 129u8, 61u8, 79u8, 217u8, 171u8, 204u8, 190u8, 40u8, 90u8, 156u8, 238u8, 120u8, 10u8, 109u8, 31u8, 137u8, 251u8, 71u8, 53u8, 163u8, 209u8, 182u8, 196u8, 82u8, 32u8, 19u8, 97u8, 247u8, 133u8, 226u8, 144u8, 6u8, 116u8, 200u8, 186u8, 44u8, 94u8, 57u8, 75u8, 221u8, 175u8, 187u8, 201u8, 95u8, 45u8, 74u8, 56u8, 174u8, 220u8, 96u8, 18u8, 132u8, 246u8, 145u8, 227u8, 117u8, 7u8, 52u8, 70u8, 208u8, 162u8, 197u8, 183u8, 33u8, 83u8, 239u8, 157u8, 11u8, 121u8, 30u8, 108u8, 250u8, 136u8, 210u8, 160u8, 54u8, 68u8, 35u8, 81u8, 199u8, 181u8, 9u8, 123u8, 237u8, 159u8, 248u8, 138u8, 28u8, 110u8, 93u8, 47u8, 185u8, 203u8, 172u8, 222u8, 72u8, 58u8, 134u8, 244u8, 98u8, 16u8, 119u8, 5u8, 147u8, 225u8, 245u8, 135u8, 17u8, 99u8, 4u8, 118u8, 224u8, 146u8, 46u8, 92u8, 202u8, 184u8, 223u8, 173u8, 59u8, 73u8, 122u8, 8u8, 158u8, 236u8, 139u8, 249u8, 111u8, 29u8, 161u8, 211u8, 69u8, 55u8, 80u8, 34u8, 180u8, 198u8];
const REF_8_B8: [u8; 256] = [0u8, 100u8, 200u8, 172u8, 225u8, 133u8, 41u8, 77u8, 179u8, 215u8, 123u8, 31u8, 82u8, 54u8, 154u8, 254u8, 23u8, 115u8, 223u8, 187u8, 246u8, 146u8, 62u8, 90u8, 164u8, 192u8, 108u8, 8u8, 69u8, 33u8, 141u8, 233u8, 46u8, 74u8, 230u8, 130u8, 207u8, 171u8, 7u8, 99u8, 157u8, 249u8, 85u8, 49u8, 124u8, 24u8, 180u8, 208u8, 57u8, 93u8, 241u8, 149u8, 216u8, 188u8, 16u8, 116u8, 138u8, 238u8, 66u8, 38u8, 107u8, 15u8, 163u8, 199u8, 92u8, 56u8, 148u8, 240u8, 189u8, 217u8, 117u8, 17u8, 239u8, 139u8, 39u8, 67u8, 14u8, 106u8, 198u8, 162u8, 75u8, 47u8, 131u8, 231u8, 170u8, 206u8, 98u8, 6u8, 248u8, 156u8, 48u8, 84u8, 25u8, 125u8, 209u8, 181u8, 114u8, 22u8, 186u8, 222u8, 147u8, 247u8, 91u8, 63u8, 193u8, 165u8, 9u8, 109u8, 32u8, 68u8, 232u8, 140u8, 101u8, 1u8, 173u8, 201u8, 132u8, 224u8, 76u8, 40u8, 214u8, 178u8, 30u8, 122u8, 55u8, 83u8, 255u8, 155u8, 184u8, 220u8, 112u8, 20u8, 89u8, 61u8, 145u8, 245u8, 11u8, 111u8, 195u8, 167u8, 234u8, 142u8, 34u8, 70u8, 175u8, 203u8, 103u8, 3u8, 78u8, 42u8, 134u8, 226u8, 28u8, 120u8, 212u8, 176u8, 253u8, 153u8, 53u8, 81u8, 150u8, 242u8, 94u8, 58u8, 119u8, 19u8, 191u8, 219u8, 37u8, 65u8, 237u8, 137u8, 196u8, 160u8, 12u8, 104u8, 129u8, 229u8, 73u8, 45u8, 96u8, 4u8, 168u8, 204u8, 50u8, 86u8, 250u8, 158u8, 211u8, 183u8, 27u8, 127u8, 228u8, 128u8, 44u8, 72u8, 5u8, 97u8, 205u8, 169u8, 87u8, 51u8, 159u8, 251u8, 182u8, 210u8, 126u8, 26u8, 243u8, 151u8, 59u8, 95u8, 18u8, 118u8, 218u8, 190u8, 64u8, 36u8, 136u8, 236u8, 161u8, 197u8, 105u8, 13u8, 202u8, 174u8, 2u8, 102u8, 43u8, 79u8, 227u8, 135u8, 121u8, 29u8, 177u8, 213u8, 152u8, 252u8, 80u8, 52u8, 221u8, 185u8, 21u8, 113u8, 60u8, 88u8, 244u8, 144u8, 110u8, 10u8, 166u8, 194u8, 143u8, 235u8, 71u8, 35u8];
const REF_8_E0: [u8; 256] = [0u8, 145u8, 227u8, 114u8, 7u8, 150u8, 228u8, 117u8, 14u8, 159u8, 237u8, 124u8, 9u8, 152u8, 234u8, 123u8, 28u8, 141u8, 255u8, 110u8, 27u8, 138u8, 248u8, 105u8, 18u8, 131u8, 241u8, 96u8, 21u8, 132u8, 246u8, 103u8, 56u8, 169u8, 219u8, 74u8, 63u8, 174u8, 220u8, 77u8, 54u8, 167u8, 213u8, 68u8, 49u8, 160u8, 210u8, 67u8, 36u8, 181u8, 199u8, 86u8, 35u8, 178u8, 192u8, 81u8, 42u8, 187u8, 201u8, 88u8, 45u8, 188u8, 206u8, 95u8, 112u8, 225u8, 147u8, 2u8, 119u8, 230u8, 148u8, 5u8, 126u8, 239u8, 157u8, 12u8, 121u8, 232u8, 154u8, 11u8, 108u8, 253u8, 143u8, 30u8, 107u8, 250u8, 136u8, 25u8, 98u8, 243u8, 129u8, 16u8, 101u8, 244u8, 134u8, 23u8, 72u8, 217u8, 171u8, 58u8, 79u8, 222u8, 172u8, 61u8, 70u8, 215u8, 165u8, 52u8, 65u8, 208u8, 162u8, 51u8, 84u8, 197u8, 183u8, 38u8, 83u8, 194u8, 176u8, 33u8, 90u8, 203u8, 185u8, 40u8, 93u8, 204u8, 190u8, 47u8, 224u8, 113u8, 3u8, 146u8, 231u8, 118u8, 4u8, 149u8, 238u8, 127u8, 13u8, 156u8, 233u8, 120u8, 10u8, 155u8, 252u8, 109u8, 31u8, 142u8, 251u8, 106u8, 24u8, 137u8, 242u8, 99u8, 17u8, 128u8, 245u8, 100u8, 22u8, 135u8, 216u8, 73u8, 59u8, 170u8, 223u8, 78u8, 60u8, 173u8, 214u8, 71u8, 53u8, 164u8, 209u8, 64u8, 50u8, 163u8, 196u8, 85u8, 39u8, 182u8, 195u8, 82u8, 32u8, 177u8, 202u8, 91u8, 41u8, 184u8, 205u8, 92u8, 46u8, 191u8, 144u8, 1u8, 115u8, 226u8, 151u8, 6u8, 116u8, 229u8, 158u8, 15u8, 125u8, 236u8, 153u8, 8u8, 122u8, 235u8, 140u8, 29u8, 111u8, 254u8, 139u8, 26u8, 104u8, 249u8, 130u8, 19u8, 97u8, 240u8, 133u8, 20u8, 102u8, 247u8, 168u8, 57u8, 75u8, 218u8, 175u8, 62u8, 76u8, 221u8, 166u8, 55u8, 69u8, 212u8, 161u8, 48u8, 66u8, 211u8, 180u8, 37u8, 87u8, 198u8, 179u8, 34u8, 80u8, 193u8, 186u8, 43u8, 89u8, 200u8, 189u8, 44u8, 94u8, 207u8];
const REF_8_D9: [u8; 256] = [0u8, 208u8, 19u8, 195u8, 38u8, 246u8, 53u8, 229u8, 76u8, 156u8, 95u8, 143u8, 106u8, 186u8, 121u8, 169u8, 152u8, 72u8, 139u8, 91u8, 190u8, 110u8, 173u8, 125u8, 212u8, 4u8, 199u8, 23u8, 242u8, 34u8, 225u8, 49u8, 131u8, 83u8, 144u8, 64u8, 165u8, 117u8, 182u8, 102u8, 207u8, 31u8, 220u8, 12u8, 233u8, 57u8, 250u8, 42u8, 27u8, 203u8, 8u8, 216u8, 61u8, 237u8, 46u8, 254u8, 87u8, 135u8, 68u8, 148u8, 113u8, 161u8, 98u8, 178u8, 181u8, 101u8, 166u8, 118u8, 147u8, 67u8, 128u8, 80u8, 249u8, 41u8, 234u8, 58u8, 223u8, 15u8, 204u8, 28u8, 45u8, 253u8, 62u8, 238u8, 11u8, 219u8, 24u8, 200u8, 97u8, 177u8, 114u8, 162u8, 71u8, 151u8, 84u8, 132u8, 54u8, 230u8, 37u8, 245u8, 16u8, 192u8, 3u8, 211u8, 122u8, 170u8, 105u8, 185u8, 92u8, 140u8, 79u8, 159u8, 174u8, 126u8, 189u8, 109u8, 136u8, 88u8, 155u8, 75u8, 226u8, 50u8, 241u8, 33u8, 196u8, 20u8, 215u8, 7u8, 217u8, 9u8, 202u8, 26u8, 255u8, 47u8, 236u8, 60u8, 149u8, 69u8, 134u8, 86u8, 179u8, 99u8, 160u8, 112u8, 65u8, 145u8, 82u8, 130u8, 103u8, 183u8, 116u8, 164u8, 13u8, 221u8, 30u8, 206u8, 43u8, 251u8, 56u8, 232u8, 90u8, 138u8, 73u8, 153u8, 124u8, 172u8, 111u8, 191u8, 22u8, 198u8, 5u8, 213u8, 48u8, 224u8, 35u8, 243u8, 194u8, 18u8, 209u8, 1u8, 228u8, 52u8, 247u8, 39u8, 142u8, 94u8, 157u8, 77u8, 168u8, 120u8, 187u8, 107u8, 108u8, 188u8, 127u8, 175u8, 74u8, 154u8, 89u8, 137u8, 32u8, 240u8, 51u8, 227u8, 6u8, 214u8, 21u8, 197u8, 244u8, 36u8, 231u8, 55u8, 210u8, 2u8, 193u8, 17u8, 184u8, 104u8, 171u8, 123u8, 158u8, 78u8, 141u8, 93u8, 239u8, 63u8, 252u8, 44u8, 201u8, 25u8, 218u8, 10u8, 163u8, 115u8, 176u8, 96u8, 133u8, 85u8, 150u8, 70u8, 119u8, 167u8, 100u8, 180u8, 81u8, 129u8, 66u8, 146u8, 59u8, 235u8, 40u8, 248u8, 29u8, 205u8, 14u8, 222u8];

impl CRCu8 {
    pub fn crc3gsm() -> CRCu8 {
        Self::create_crc(0x03, 3, 0x00, 0x07, false)
    }

    pub fn crc4itu() -> CRCu8 {
        Self::create_crc(0x0C, 4, 0x00, 0x00, true)
    }

    pub fn crc4interlaken() -> CRCu8 {
        Self::create_crc(0x03, 4, 0x0F, 0x0F, false)
    }

    pub fn crc5epc() -> CRCu8 {
        Self::create_crc(0x09, 5, 0x00, 0x00, false)
    }

    pub fn crc5itu() -> CRCu8 {
        Self::create_crc(0x15, 5, 0x00, 0x00, true)
    }

    pub fn crc5usb() -> CRCu8 {
        Self::create_crc(0x14, 5, 0x1F, 0x1F, true)
    }

    pub fn crc6cdma2000_a() -> CRCu8 {
        Self::create_crc(0x27, 6, 0x3f, 0x00, false)
    }

    pub fn crc6cdma2000_b() -> CRCu8 {
        Self::create_crc(0x07, 6, 0x3f, 0x00, false)
    }

    pub fn crc6darc() -> CRCu8 {
        Self::create_crc(0x26, 6, 0x00, 0x00, true)
    }

    pub fn crc6gsm() -> CRCu8 {
        Self::create_crc(0x2F, 6, 0x00, 0x3F, false)
    }

    pub fn crc6itu() -> CRCu8 {
        Self::create_crc(0x30, 6, 0x00, 0x00, true)
    }

    pub fn crc7() -> CRCu8 {
        Self::create_crc(0x09, 7, 0x00, 0x00, false)
    }

    pub fn crc7umts() -> CRCu8 {
        Self::create_crc(0x45, 7, 0x00, 0x00, false)
    }

    pub fn crc8() -> CRCu8 {
        // Self::create_crc(0x07, 8, 0x00, 0x00, false)

        let lookup_table = NO_REF_8_07;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, false)
    }

    pub fn crc8cdma2000() -> CRCu8 {
        // Self::create_crc(0x9B, 8, 0xFF, 0x00, false)

        let lookup_table = NO_REF_8_9B;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, false)
    }

    pub fn crc8darc() -> CRCu8 {
//        Self::create_crc(0x9C, 8, 0x00, 0x00, true)

        let lookup_table = REF_8_9C;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }

    pub fn crc8dvb_s2() -> CRCu8 {
//        Self::create_crc(0xD5, 8, 0x00, 0x00, false)

        let lookup_table = NO_REF_8_D5;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, false)
    }

    pub fn crc8ebu() -> CRCu8 {
//        Self::create_crc(0xB8, 8, 0xFF, 0x00, true)

        let lookup_table = REF_8_B8;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, true)
    }

    pub fn crc8icode() -> CRCu8 {
//        Self::create_crc(0x1D, 8, 0xFD, 0x00, false)

        let lookup_table = NO_REF_8_1D;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFD, 0x00, false)
    }

    pub fn crc8itu() -> CRCu8 {
//        Self::create_crc(0x07, 8, 0x00, 0x55, false)

        let lookup_table = NO_REF_8_07;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x55, false)
    }

    pub fn crc8maxim() -> CRCu8 {
//        Self::create_crc(0x8C, 8, 0x00, 0x00, true)

        let lookup_table = REF_8_8C;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }

    pub fn crc8rohc() -> CRCu8 {
//        Self::create_crc(0xE0, 8, 0xFF, 0x00, true)

        let lookup_table = REF_8_E0;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0xFF, 0x00, true)
    }

    pub fn crc8wcdma() -> CRCu8 {
//        Self::create_crc(0xD9, 8, 0x00, 0x00, true)

        let lookup_table = REF_8_D9;
        Self::create_crc_with_exists_lookup_table(lookup_table, 8, 0x00, 0x00, true)
    }
}

#[cfg(all(feature = "development", not(feature = "no_std"), test))]
mod tests {
    use super::CRCu8;

    use std::fmt::Write;

    #[test]
    fn print_lookup_table() {
        let crc = CRCu8::crc4interlaken();

        let mut s = String::new();

        for n in crc.lookup_table.iter().take(255) {
            s.write_fmt(format_args!("{}u8, ", n)).unwrap();
        }

        s.write_fmt(format_args!("{}u8", crc.lookup_table[255])).unwrap();

        println!("let lookup_table = [{}];", s);
    }
}
