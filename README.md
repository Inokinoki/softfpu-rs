# Software-emulated FPU in Rust

Use 32-bit unsigned integer to represent Float32 in [IEEE-754](https://en.wikipedia.org/wiki/IEEE_754) and do float calculation.

# Usage

There is an operator-trait-based API:

## Add

```rust
let v0_1 = soft_f32::F32::from_u32(0x3DCCCCCD);  // 0.1
let v0_2 = soft_f32::F32::from_u32(0x3E4CCCCD);  // 0.2

let v0_3 = v0_1 + v0_2; // 0.1 + 0.2

assert_eq!(v0_3.value(), 0x3E99999A);
```

and a procedure-style API:

```rust
let v0_1 = 0x3DCCCCCD;  // 0.1
let v0_2 = 0x3E4CCCCD;  // 0.2

let v0_3 = soft_f32::f32_add(v0_1, v0_2); // 0.1 + 0.2

assert_eq!(v0_3, 0x3E99999A);
```

## Subtract

```rust
let v0_1 = soft_f32::F32::from_u32(0x3DCCCCCD);  // 0.1
let v0_2 = soft_f32::F32::from_u32(0x3E4CCCCD);  // 0.2

let v0_1_result = v0_2 - v0_1;  // 0.2 - 0.1

assert_eq!(v0_1_result.value(), 0x3DCCCCCD);
```

## Multiply

```rust
let v0_1 = soft_f32::F32::from_u32(0x3DCCCCCD);  // 0.1
let v0_2 = soft_f32::F32::from_u32(0x3E4CCCCD);  // 0.2

let v0_02 = v0_2 * v0_1;    // 0.2 * 0.1

assert_eq!(v0_02.value(), 0x3CA3D70B);
```

## Division

```rust
let v0_1 = soft_f32::F32::from_u32(0x3DCCCCCD);  // 0.1
let v0_2 = soft_f32::F32::from_u32(0x3E4CCCCD);  // 0.2

let v0_5 = v0_1 / v0_2; // 0.1 / 0.2

assert_eq!(v0_5.value(), 0x3F000000);
```

## Squared-root

```rust
let v0_01 = crate::soft_f32::F32::from_u32(0x3C23D70A); // 0.01

let v0_1 = v0_01.sqrt();    // sqrt(0.01)

assert_eq!(v0_1.value(), 0x3DCCCCCD);
```

## Comparison

```rust
let v0_1 = 0x3DCCCCCD;  // 0.1
let v0_2 = 0x3E4CCCCD;  // 0.2

assert_eq!(v0_1 < v0_2, true);
assert_eq!(v0_1 <= v0_2, true);
assert_eq!(v0_1 < v0_1, false);
assert_eq!(v0_1 <= v0_1, true);
assert_eq!(v0_1 == v0_1, true);
assert_eq!(v0_1 != v0_1, false);
```

# Development

Currently only aiming at implementing Float32.

## TODOs

- [ ] Publish on crate.io
- [ ] Float32 Log2 (v0.2.X)
- [ ] Float32 Exp (v0.2.X)
- [ ] Float32 Sin, Cos (v0.2.X)
- [ ] Float80 (v0.3.X)

# Conclusion

It is a by-product of one of my projects to bring a float number subsystem into a PL that is not capable of handling float numbers. And finally, implement a more complex system.
