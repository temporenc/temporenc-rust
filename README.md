[![](https://img.shields.io/crates/v/temporenc.svg)](https://crates.io/crates/temporenc) [![](https://docs.rs/temporenc/badge.svg)](https://docs.rs/temporenc/) 

Rust library for [Temporenc](https://temporenc.org/), a binary date/time format.

Great performance is a high priority for this implementation. It uses a different struct for each of the temporal types (date, date and time, etc). This allows for hand-tuned serialization and deserialization logic for each type. There also is no heap allocation (aside from test code, of course).

See below for some sample performance numbers from an i7-6850K (a 3.6Ghz Broadwell-E chip). Batches of 100 are used because time measurement accuracy is poor when single operations only take a few nanoseconds.

| Operation | Quantity | Type | Time |
|-----------|----------|------|------|
| Deserialize | 100 | random date | 665ns |
| Deserialize | 100 | random date + time | 985ns |
| Deserialize | 100 | random date + time + offset | 974ns |
| Deserialize | 100 | random date + time + subsecond | 1300ns |
| Deserialize | 100 | random date + time + subsecond + offset | 1473ns |
| Deserialize | 100 | random time | 801ns |
| Serialize | 100 | random date | 288ns |
| Serialize | 100 | random date + time | 435ns |
| Serialize | 100 | random date + time + offset | 492ns |
| Serialize | 100 | random date + time + subsecond | 897ns |
| Serialize | 100 | random date + time + subsecond + offset | 982ns |
| Serialize | 100 | random time | 277ns |

