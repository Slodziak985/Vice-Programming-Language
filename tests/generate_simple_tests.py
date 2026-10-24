import os

base_dir = r"c:\Users\Slodziak985\Desktop\Vice 0.1.1\tests\programs"

def ensure_dirs():
    for d in ["simple", "medium", "heavy"]:
        os.makedirs(os.path.join(base_dir, d), exist_ok=True)

# -------------------------------------------------------------
# 100 SIMPLE PROGRAMS (~20 lines each)
# -------------------------------------------------------------
def generate_simple():
    simple_dir = os.path.join(base_dir, "simple")
    for i in range(1, 101):
        num_str = f"{i:03d}"
        path = os.path.join(simple_dir, f"test_{num_str}.vice")
        
        # We craft distinct algorithmic test programs for each number:
        if i == 1:
            code = """# Test 001: Basic arithmetic and precedence
fn main() {
    mut a = 10;
    mut b = 25;
    mut c = a + b * 2 - (15 / 3);
    # 10 + 50 - 5 = 55
    if c == 55 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 2:
            code = """# Test 002: Floating point computation
fn main() {
    mut x = 12.5;
    mut y = 2.5;
    mut res = (x / y) + 5.0;
    # 5.0 + 5.0 = 10.0
    if res > 9.99 and res < 10.01 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 3:
            code = """# Test 003: String concatenation and length
fn main() {
    mut s1 = "Hello, ";
    mut s2 = "Vice!";
    mut full = s1 + s2;
    mut l = string.len(full);
    if full == "Hello, Vice!" and l == 12 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 4:
            code = """# Test 004: Array push, pop and length
fn main() {
    mut arr = [10, 20, 30];
    arr = array.push(arr, 40);
    mut l = array.len(arr);
    mut last = array.last(arr);
    if l == 4 and last == 40 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 5:
            code = """# Test 005: While loop accumulator
fn main() {
    mut sum = 0;
    mut i = 1;
    while i <= 10 {
        sum = sum + i;
        i = i + 1;
    }
    # Sum of 1..10 is 55
    if sum == 55 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 6:
            code = """# Test 006: For-in loop over array
fn main() {
    mut items = [2, 4, 6, 8, 10];
    mut product = 1;
    for item in items {
        product = product * item;
    }
    # 2*4*6*8*10 = 3840
    if product == 3840 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 7:
            code = """# Test 007: Map insertion and lookup
fn main() {
    mut m = {"k1": 100, "k2": 200};
    mut v1 = map.get(m, "k1", 0);
    mut v2 = map.get(m, "k2", 0);
    mut v3 = map.get(m, "k3", -1);
    if v1 == 100 and v2 == 200 and v3 == -1 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 8:
            code = """# Test 008: Recursive factorial function
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    mut fact6 = factorial(6);
    if fact6 == 720 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 9:
            code = """# Test 009: Recursive Fibonacci
fn fib(n: int) -> int {
    if n <= 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    mut f7 = fib(7);
    # Fib(7) is 13
    if f7 == 13 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        elif i == 10:
            code = """# Test 010: Math square root and power
fn main() {
    mut sq = math.sqrt(144.0);
    mut pw = math.pow(2.0, 8.0);
    if sq > 11.99 and sq < 12.01 and pw > 255.99 and pw < 256.01 {
        output <- "PASS";
    } else {
        output <- "FAIL";
    }
}
"""
        else:
            # Procedurally generate robust, meaningful tests for 11..100
            offset = i
            domain_idx = (i - 11) % 10
            if domain_idx == 0:
                # Math series
                code = f"""# Test {num_str}: Mathematical series step {i}
fn main() {{
    mut total = 0;
    mut k = 1;
    while k <= 5 {{
        total = total + (k * k + {offset});
        k = k + 1;
    }}
    # 1+4+9+16+25 = 55 + 5*{offset}
    mut expected = 55 + (5 * {offset});
    if total == expected {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 1:
                # Array manipulation
                code = f"""# Test {num_str}: Array filtering and transformation {i}
fn main() {{
    mut src = [{i}, {i+1}, {i+2}, {i+3}, {i+4}];
    mut doubled = [];
    for item in src {{
        doubled = array.push(doubled, item * 2);
    }}
    mut first = array.first(doubled);
    mut last = array.last(doubled);
    if first == {2 * i} and last == {2 * (i + 4)} {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 2:
                # String processing
                code = f"""# Test {num_str}: String transformations {i}
fn main() {{
    mut s = "vice_test_case_{num_str}";
    mut upper = string.to_upper(s);
    mut has_sub = string.contains(upper, "TEST");
    mut l = string.len(upper);
    if has_sub and l > 10 {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 3:
                # Map frequency counter
                code = f"""# Test {num_str}: Frequency counting in map {i}
fn main() {{
    mut m = {{}};
    m = map.set(m, "count_{i}", {i * 10});
    m = map.set(m, "multiplier", 3);
    mut count = map.get(m, "count_{i}", 0);
    mut mult = map.get(m, "multiplier", 1);
    mut res = count * mult;
    if res == {i * 30} {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 4:
                # Crypto SHA256 test
                code = f"""# Test {num_str}: Crypto SHA256 integrity check {i}
fn main() {{
    mut h = crypto.sha256("vice_input_data_{i}");
    mut l = string.len(h);
    mut b64 = crypto.base64_encode("test_{i}");
    mut decoded = crypto.base64_decode(b64);
    if l == 64 and decoded == "test_{i}" {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 5:
                # Matrix algebra test
                code = f"""# Test {num_str}: Matrix operations {i}
fn main() {{
    mut m1 = matrix.zeros(2, 2);
    mut arr_data = [{float(i)}, 0.0, 0.0, {float(i)}];
    mut m2 = matrix.from_array(arr_data, 2, 2);
    mut shape = matrix.shape(m2);
    mut sum_val = matrix.sum(m2);
    if shape[0] == 2 and shape[1] == 2 and sum_val > {float(2*i - 1)} {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 6:
                # Regex match test
                code = f"""# Test {num_str}: Regex validation {i}
fn main() {{
    mut text = "order_{i}_code_ABC";
    mut ok1 = regex.is_match("order_\\\\d+_code_[A-Z]+", text);
    mut ok2 = regex.is_match("^\\\\d+$", text);
    mut rep = regex.replace("_code_ABC", text, "_DONE");
    if ok1 and !ok2 and rep == "order_{i}_DONE" {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 7:
                # Custom Data structure
                code = f"""# Test {num_str}: Custom data structure {i}
data Box{i} {{
    width: int,
    height: int,
    depth: int
}}

fn volume(b: Box{i}) -> int {{
    return b.width * b.height * b.depth;
}}

fn main() {{
    mut b = Box{i}({i % 10 + 2}, 3, 4);
    mut v = volume(b);
    if v == {(i % 10 + 2) * 12} {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            elif domain_idx == 8:
                # SQLite in-memory test
                code = f"""# Test {num_str}: SQLite in-memory execution {i}
fn main() {{
    mut db = sqlite.open(":memory:");
    sqlite.execute(db, "CREATE TABLE t (val INT);", []);
    sqlite.execute(db, "INSERT INTO t VALUES (?);", [{i * 7}]);
    mut rows = sqlite.query(db, "SELECT val FROM t;", []);
    sqlite.close(db);
    if array.len(rows) == 1 {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""
            else:
                # Logic and bitwise simulation
                code = f"""# Test {num_str}: Logical assertions {i}
fn is_even(n: int) -> bool {{
    return (n % 2) == 0;
}}

fn main() {{
    mut num = {i};
    mut even = is_even(num);
    mut check = (num % 2 == 0);
    if even == check {{
        output <- "PASS";
    }} else {{
        output <- "FAIL";
    }}
}}
"""

        with open(path, "w", encoding="utf-8") as f:
            f.write(code)

    print("Generated 100 simple tests.")

ensure_dirs()
generate_simple()
