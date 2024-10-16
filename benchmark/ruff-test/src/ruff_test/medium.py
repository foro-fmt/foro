import random
import string


# Utility functions
def generate_random_string(length: int) -> str:
    return "".join(random.choices(string.ascii_letters + string.digits, k=length))


def generate_random_numbers(count: int) -> list:
    return [random.randint(0, 1000) for _ in range(count)]


def write_to_file(filename: str, content: str) -> None:
    with open(filename, "w") as f:
        f.write(content)


def read_from_file(filename: str) -> str:
    with open(filename, "r") as f:
        return f.read()


# Simple class with methods
class DataProcessor:
    def __init__(self, data: list):
        self.data = data

    def filter_data(self, threshold: int) -> list:
        return [x for x in self.data if x > threshold]

    def sort_data(self) -> list:
        return sorted(self.data)

    def save_data(self, filename: str) -> None:
        content = "\n".join(map(str, self.data))
        write_to_file(filename, content)

    def load_data(self, filename: str) -> None:
        content = read_from_file(filename)
        self.data = list(map(int, content.splitlines()))


# Generate a list of random strings
def generate_data(size: int) -> list:
    data = []
    for _ in range(size):
        data.append(generate_random_string(random.randint(5, 15)))
    return data


# Find the longest string in a list
def find_longest_string(data: list) -> str:
    return max(data, key=len)


# Process numbers and strings
def process_numbers_and_strings():
    numbers = generate_random_numbers(200)
    processor = DataProcessor(numbers)
    filtered_numbers = processor.filter_data(500)
    sorted_numbers = processor.sort_data()
    processor.save_data("numbers.txt")

    strings = generate_data(50)
    longest_string = find_longest_string(strings)

    print("Longest string:", longest_string)
    print("Top 10 sorted numbers:", sorted_numbers[:10])


# A recursive function to calculate factorial
def factorial(n: int) -> int:
    if n == 0:
        return 1
    return n * factorial(n - 1)


# A simple prime number checker
def is_prime(n: int) -> bool:
    if n < 2:
        return False
    for i in range(2, int(n**0.5) + 1):
        if n % i == 0:
            return False
    return True


# Finding primes in a range
def find_primes(limit: int) -> list:
    primes = []
    for num in range(2, limit):
        if is_prime(num):
            primes.append(num)
    return primes


# Sorting words by their length
def sort_words_by_length(words: list) -> list:
    return sorted(words, key=len)


# Main function to demonstrate various operations
def main():
    # Processing numbers and strings
    process_numbers_and_strings()

    # Factorial calculation
    num = 10
    fact_result = factorial(num)
    print(f"Factorial of {num}: {fact_result}")

    # Prime number calculation
    prime_limit = 100
    primes = find_primes(prime_limit)
    print(f"Primes up to {prime_limit}: {primes}")

    # Sorting words
    words = ["apple", "banana", "cherry", "date", "elderberry", "fig", "grape"]
    sorted_words = sort_words_by_length(words)
    print("Words sorted by length:", sorted_words)


# Extended DataProcessor with additional functionality
class ExtendedDataProcessor(DataProcessor):
    def __init__(self, data: list):
        super().__init__(data)

    def find_max(self) -> int:
        if not self.data:
            raise ValueError("Data is empty")
        return max(self.data)

    def find_min(self) -> int:
        if not self.data:
            raise ValueError("Data is empty")
        return min(self.data)

    def calculate_average(self) -> float:
        if not self.data:
            raise ValueError("Data is empty")
        return sum(self.data) / len(self.data)


# Simple matrix operations
class Matrix:
    def __init__(self, rows: int, cols: int):
        self.rows = rows
        self.cols = cols
        self.matrix = [
            [random.randint(0, 10) for _ in range(cols)] for _ in range(rows)
        ]

    def display(self) -> None:
        for row in self.matrix:
            print(row)

    def transpose(self) -> list:
        return [[self.matrix[j][i] for j in range(self.rows)] for i in range(self.cols)]

    def add(self, other_matrix: "Matrix") -> list:
        if self.rows != other_matrix.rows or self.cols != other_matrix.cols:
            raise ValueError("Matrices must have the same dimensions for addition")
        return [
            [self.matrix[i][j] + other_matrix.matrix[i][j] for j in range(self.cols)]
            for i in range(self.rows)
        ]


# Matrix usage demonstration
def matrix_operations():
    matrix1 = Matrix(3, 3)
    matrix2 = Matrix(3, 3)

    print("Matrix 1:")
    matrix1.display()

    print("\nMatrix 2:")
    matrix2.display()

    print("\nSum of Matrix 1 and Matrix 2:")
    result = matrix1.add(matrix2)
    for row in result:
        print(row)

    print("\nTranspose of Matrix 1:")
    transposed = matrix1.transpose()
    for row in transposed:
        print(row)


# Fibonacci series calculation (Iterative)
def fibonacci(n: int) -> int:
    if n <= 0:
        raise ValueError("Input must be a positive integer")
    if n == 1:
        return 0
    elif n == 2:
        return 1
    else:
        a, b = 0, 1
        for _ in range(n - 2):
            a, b = b, a + b
        return b


# Generate Fibonacci series up to n terms
def fibonacci_series(n: int) -> list:
    series = []
    for i in range(1, n + 1):
        series.append(fibonacci(i))
    return series


# Main function to demonstrate extended functionalities
def main_extended():
    # DataProcessor demonstration
    numbers = generate_random_numbers(100)
    ext_processor = ExtendedDataProcessor(numbers)
    print("Max number:", ext_processor.find_max())
    print("Min number:", ext_processor.find_min())
    print("Average of numbers:", ext_processor.calculate_average())

    # Matrix operations
    matrix_operations()

    # Fibonacci series
    n_terms = 10
    fib_series = fibonacci_series(n_terms)
    print(f"First {n_terms} terms of Fibonacci series:", fib_series)


# Simple string manipulations
def string_manipulations():
    strings = generate_data(20)
    upper_strings = [s.upper() for s in strings]
    reversed_strings = [s[::-1] for s in strings]

    print("Original strings:", strings)
    print("Uppercase strings:", upper_strings)
    print("Reversed strings:", reversed_strings)


# Execute extended main functionality
if __name__ == "__main__":
    main()
    main_extended()
    string_manipulations()
