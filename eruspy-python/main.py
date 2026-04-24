"""
Rust 함수를 Python에서 호출하는 예제.

빌드 방법:
    pip install maturin
    cd eruspy-python
    maturin develop          # 개발용 (현재 venv에 설치)
    # 또는
    maturin build --release  # 배포용 .whl 생성
"""

import eruspy_python as rust

# ─────────────────────────────────────────────────────────────────────────────
# 정수 / 부동소수점  →  int / float
# ─────────────────────────────────────────────────────────────────────────────
print("=== 정수 / 부동소수점 ===")
print(rust.add_ints(3, 4))           # 7
print(rust.add_floats(1.5, 2.3))     # 3.8
print(rust.factorial(10))            # 3628800

# ─────────────────────────────────────────────────────────────────────────────
# 불리언  →  bool
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 불리언 ===")
print(rust.negate(True))             # False
print(rust.is_prime(97))             # True
print(rust.is_prime(100))            # False

# ─────────────────────────────────────────────────────────────────────────────
# 문자열  →  str
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 문자열 ===")
print(rust.greet("세계"))            # Hello, 세계!
print(rust.reverse_str("Rust"))      # tsuR
print(rust.split_by("a,b,c", ","))   # ['a', 'b', 'c']

# ─────────────────────────────────────────────────────────────────────────────
# Vec  →  list
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 리스트 ===")
print(rust.range_vec(5))                          # [0, 1, 2, 3, 4]
print(rust.word_lengths(["hello", "world", "!"]))  # [5, 5, 1]
print(rust.squares([1.0, 2.0, 3.0]))              # [1.0, 4.0, 9.0]
print(rust.sorted_vec([3.0, 1.0, 2.0]))           # [1.0, 2.0, 3.0]

# ─────────────────────────────────────────────────────────────────────────────
# HashMap  →  dict
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 딕셔너리 ===")
print(rust.count_chars("hello"))                           # {'h':1,'e':1,'l':2,'o':1}
print(rust.group_by_first_char(["apple","ant","banana","cherry","cat"]))
print(rust.merge_dicts({"a": 1, "b": 2}, {"b": 3, "c": 4}))  # {'a':1,'b':5,'c':4}

# ─────────────────────────────────────────────────────────────────────────────
# 튜플  →  tuple
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 튜플 ===")
mn, mx = rust.min_max([3.0, 1.0, 4.0, 1.0, 5.0])
print(f"min={mn}, max={mx}")         # min=1.0, max=5.0

total, avg = rust.sum_and_mean([1.0, 2.0, 3.0, 4.0, 5.0])
print(f"sum={total}, mean={avg}")    # sum=15.0, mean=3.0

# ─────────────────────────────────────────────────────────────────────────────
# Option  →  값 또는 None
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== Option (None 가능) ===")
print(rust.safe_divide(10.0, 3.0))   # 3.333...
print(rust.safe_divide(10.0, 0.0))   # None

print(rust.find_first_even([1, 3, 5, 4, 7]))  # 4
print(rust.find_first_even([1, 3, 5]))         # None

print(rust.max_of([5, 2, 8, 1]))    # 8
print(rust.max_of([]))              # None

# ─────────────────────────────────────────────────────────────────────────────
# Result  →  값 또는 예외
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== Result (예외 가능) ===")
print(rust.parse_int("42"))          # 42
try:
    rust.parse_int("not_a_number")
except ValueError as e:
    print(f"parse_int 에러: {e}")

print(rust.safe_sqrt(9.0))           # 3.0
try:
    rust.safe_sqrt(-1.0)
except ValueError as e:
    print(f"safe_sqrt 에러: {e}")

print(rust.is_valid_json('{"key": [1, 2, 3]}'))  # True
print(rust.is_valid_json('{broken'))              # False

# ─────────────────────────────────────────────────────────────────────────────
# 커스텀 struct  →  Python 객체
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 커스텀 struct (Point) ===")
p1 = rust.Point(0.0, 0.0)
p2 = rust.Point(3.0, 4.0)

print(p1)                            # Point(0, 0)
print(p2)                            # Point(3, 4)
print(p2.distance_to(p1))           # 5.0
print(p2.as_tuple())                # (3.0, 4.0)

p3 = p2.translate(1.0, -1.0)
print(p3)                            # Point(4, 3)
print(p3.x, p3.y)                   # 속성 직접 접근

points = [rust.Point(1.0, 0.0), rust.Point(3.0, 4.0), rust.Point(-1.0, -1.0)]
print(rust.farthest_from_origin(points))  # Point(3, 4)

# ─────────────────────────────────────────────────────────────────────────────
# 중첩 컬렉션  →  list[list[int]]
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== 중첩 컬렉션 ===")
matrix = [[1, 2, 3], [4, 5, 6]]
print(rust.transpose(matrix))        # [[1, 4], [2, 5], [3, 6]]
