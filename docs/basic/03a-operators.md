

## Arithmetic 

Used for basic math:

| Operator | Meaning        | Example  |
| -------- | -------------- | -------- |
|  +       | Addition       |  3 + 2   |
|  -       | Subtraction    |  10 - 4  |
|  *       | Multiplication |  4 * 2   |
|  /       | Division       |  10 / 3  |
|  %       | Modulus        |  10 % 3  |

```rust
fn main() {
    let a = 10;
    let b = 3;

    println!("Add: {}", a + b);
    println!("Modulus: {}", a % b);
}
```
   


## Comparison 

Used in conditions; result is  true  or  false .

| Operator | Meaning          | Example  |
| -------- | ---------------- | -------- |
|  ==      | Equal to         |  a == b  |
|  !=      | Not equal to     |  a != b  |
|  >       | Greater than     |  a > b   |
|  <       | Less than        |  a < b   |
|  >=      | Greater or equal |  a >= b  |
|  <=      | Less or equal    |  a <= b  |

```rust
let x = 5;
let y = 10;
println!("{}", x < y); // true
```   


## Logical 

Work with boolean values:

| Operator | Meaning | Example            |  
|------|---------|--------------------| 
| &&   | AND     | a > 5 && a < 10    |    
| \|\| | OR | a == 0 \|\| a == 1 |
| !    | NOT     | !true  →  false    |   

```rust
let x = 7;
println!("{}", x > 5 && x < 10); // true
```   


## Bitwise 

Operate on binary values (advanced use cases):


| Operator | Meaning     | Example |
|----------|-------------|---------|
| \&       | AND         | a \& b  |
| \|       | OR          | a \| b  |
| \^       | XOR         | a \^ b  |
| <<       | Left Shift  | a << 1  |
| >>       | Right Shift | a >> 1  |


```rust
let a = 0b1100;
let b = 0b1010;
println!("{:b}", a & b); // 1000
```   


## Assignment 

| Operator | Meaning             | Example  |
| -------- | ------------------- | -------- |
|  =       | Assign              |  x = 5   |
|  +=      | Add and assign      |  x += 2  |
|  -=      | Subtract and assign |  x -= 1  |
|  *=      | Multiply and assign |  x *= 3  |
|  /=      | Divide and assign   |  x /= 2  |
|  %=      | Modulo and assign   |  x %= 2  |

```rust
let mut x = 5;
x += 3; // x is now 8
```
   


## Range 

| Syntax  | Description     | Example          |
| ------- | --------------- | ---------------- |
|  a..b   | Exclusive range |  1..5  → 1 to 4  |
|  a..=b  | Inclusive range |  1..=5  → 1 to 5 |

```rust
for i in 1..4 {
    println!("{}", i); // 1, 2, 3
}
```
   


## Summary

| Category   | Examples            |       
|------------|---------------------|
| Arithmetic | + ,  - ,  * ,  / ,  % |   
| Comparison | == ,  != ,  < ,  >  |   
| Logical    | && ,  \|\|  ,  !    |
| Bitwise    | & , \| ,  ^ ,  <<   |   
| Assignment | = ,  += ,  *=       |        
| Range      | 1..5 ,  1..=5       |     