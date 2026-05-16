from typing import Callable, List, Union
from dataclasses import dataclass
from functools import wraps


# Type aliases
Token = Union[str, List]
CommandStack = List[Token]


# ==================== Tokenization ====================


def split_input(input_str: str) -> List[Token]:
    """Parse input string into tokens, handling nested parentheses."""
    tokens = []
    current_token = ""
    paren_stack = []

    for char in input_str:
        if char == "(":
            if current_token:
                tokens.append(current_token)
                current_token = ""
            paren_stack.append([])
        elif char == ")":
            if current_token:
                paren_stack[-1].append(current_token)
                current_token = ""
            closed = paren_stack.pop()
            if paren_stack:
                paren_stack[-1].append(closed)
            else:
                tokens.append(closed)
        elif char.isspace():
            if current_token:
                if paren_stack:
                    paren_stack[-1].append(current_token)
                else:
                    tokens.append(current_token)
                current_token = ""
        else:
            current_token += char

    if current_token:
        tokens.append(current_token)
    return tokens


# ==================== Type Checking ====================


def is_es(value: Token) -> bool:
    """Check if value is an executable structure (list)."""
    return isinstance(value, list)


def is_number(value: Token) -> bool:
    """Check if value is a number (string)."""
    return not is_es(value)


# ==================== Validation ====================


class PostfixError(Exception):
    """Base exception for postfix evaluation errors."""

    pass


def check_operands(
    commands: CommandStack, index: int, count: int, allow_es: bool = False
) -> None:
    """Validate that enough operands exist and are of correct type."""
    if index < count:
        raise PostfixError("Not enough operands")

    if not allow_es:
        for i in range(1, count + 1):
            if is_es(commands[index - i]):
                raise PostfixError("Not Numbers")


def check_divisor(b: int) -> None:
    """Validate divisor is not zero."""
    if b == 0:
        raise PostfixError("Division by zero")


# ==================== Command Handlers ====================


@dataclass
class CommandHandler:
    """Encapsulates command validation and execution logic."""

    operand_count: int
    allow_es: bool
    execute: Callable[[CommandStack, int], int]


def binary_operation(
    commands: CommandStack, index: int, op: Callable[[int, int], int]
) -> int:
    """Execute a binary operation (add, sub, mul, div, rem)."""
    check_operands(commands, index, 2)

    b = int(commands[index - 1])
    a = int(commands[index - 2])
    commands[index] = str(op(a, b))
    del commands[index - 2 : index]
    return index - 1


def comparison_operation(
    commands: CommandStack, index: int, op: Callable[[int, int], bool]
) -> int:
    """Execute a comparison operation (lt, gt, eq)."""
    check_operands(commands, index, 2)

    b = int(commands[index - 1])
    a = int(commands[index - 2])
    commands[index] = "1" if op(a, b) else "0"
    del commands[index - 2 : index]
    return index - 1


def cmd_add(commands: CommandStack, index: int) -> int:
    return binary_operation(commands, index, lambda a, b: a + b)


def cmd_sub(commands: CommandStack, index: int) -> int:
    return binary_operation(commands, index, lambda a, b: a - b)


def cmd_mul(commands: CommandStack, index: int) -> int:
    return binary_operation(commands, index, lambda a, b: a * b)


def cmd_div(commands: CommandStack, index: int) -> int:
    check_operands(commands, index, 2)
    b = int(commands[index - 1])
    check_divisor(b)
    a = int(commands[index - 2])
    commands[index] = str(a // b)
    del commands[index - 2 : index]
    return index - 1


def cmd_rem(commands: CommandStack, index: int) -> int:
    check_operands(commands, index, 2)
    b = int(commands[index - 1])
    check_divisor(b)
    a = int(commands[index - 2])
    commands[index] = str(a % b)
    del commands[index - 2 : index]
    return index - 1


def cmd_sel(commands: CommandStack, index: int) -> int:
    """Select between two values based on condition."""
    check_operands(commands, index, 3)

    condition = commands[index - 3]
    true_val = commands[index - 2]
    false_val = commands[index - 1]
    commands[index] = false_val if condition == "0" else true_val
    del commands[index - 3 : index]
    return index - 3


def cmd_nget(commands: CommandStack, index: int) -> int:
    """Get element at relative index."""
    check_operands(commands, index, 1)

    i = int(commands[index - 1])
    if i < 0 or i >= len(commands):
        raise PostfixError("Index out of bounds")

    commands[index] = commands[index - 1 - i]
    del commands[index - 1]
    return index - 1


def cmd_lt(commands: CommandStack, index: int) -> int:
    return comparison_operation(commands, index, lambda a, b: a < b)


def cmd_gt(commands: CommandStack, index: int) -> int:
    return comparison_operation(commands, index, lambda a, b: a > b)


def cmd_eq(commands: CommandStack, index: int) -> int:
    return comparison_operation(commands, index, lambda a, b: a == b)


def cmd_swap(commands: CommandStack, index: int) -> int:
    """Swap top two elements."""
    check_operands(commands, index, 2, allow_es=True)

    commands[index - 1], commands[index - 2] = commands[index - 2], commands[index - 1]
    del commands[index]
    return index - 1


def cmd_pop(commands: CommandStack, index: int) -> int:
    """Remove top element."""
    check_operands(commands, index, 1, allow_es=True)

    del commands[index - 1 : index + 1]
    return index - 2


def cmd_exec(commands: CommandStack, index: int) -> int:
    """Execute an executable structure."""
    check_operands(commands, index, 1, allow_es=True)

    es = commands[index - 1]
    if not is_es(es):
        raise PostfixError("Not an ES")

    commands[index - 1 : index + 1] = es
    return index - 1


# ==================== Command Registry ====================

COMMAND_HANDLERS: dict[str, CommandHandler] = {
    "add": CommandHandler(2, False, cmd_add),
    "sub": CommandHandler(2, False, cmd_sub),
    "mul": CommandHandler(2, False, cmd_mul),
    "div": CommandHandler(2, False, cmd_div),
    "rem": CommandHandler(2, False, cmd_rem),
    "sel": CommandHandler(3, False, cmd_sel),
    "nget": CommandHandler(1, False, cmd_nget),
    "lt": CommandHandler(2, False, cmd_lt),
    "gt": CommandHandler(2, False, cmd_gt),
    "eq": CommandHandler(2, False, cmd_eq),
    "swap": CommandHandler(2, True, cmd_swap),
    "pop": CommandHandler(1, True, cmd_pop),
    "exec": CommandHandler(1, True, cmd_exec),
}


# ==================== Debug Output ====================


def format_commands(commands: CommandStack) -> str:
    """Format commands list for display."""
    return (
        str(commands)
        .replace("[", "(")
        .replace("]", ")")
        .replace("'", "")
        .replace(", ", " ")
    )


def print_state(arg_count: str, commands: CommandStack, index: int) -> None:
    """Print current execution state."""
    prev = format_commands(commands[: index + 1])[1:-1]
    next_part = format_commands(commands[index + 1 :])[1:-1]
    print(f"(postfix {arg_count} {prev} | {next_part})")


# ==================== Main Interpreter ====================


def main() -> None:
    input_str = "(postfix 3 1 nget 3 nget lt 2 nget 4 nget sel swap pop swap pop 1 nget 3 nget lt 2 nget 4 nget sel swap pop swap pop)"
    argv = "1 2 3"

    # Validate input format
    if not input_str.startswith("(postfix") or not input_str.endswith(")"):
        print("Invalid input format")
        return

    # Parse input
    input_str = input_str[8:-1]  # Remove "(postfix " and ")"
    arg_count, *commands = split_input(input_str)

    # Validate argument count
    argv_list = argv.split()
    if int(arg_count) != len(argv_list):
        print("Argument count does not match")
        return

    # Prepend arguments reversed
    commands = argv_list[::-1] + commands

    # Execute commands
    index = 0
    try:
        while index < len(commands):
            if not commands[index]:
                break

            token = commands[index]
            if token not in COMMAND_HANDLERS:
                index += 1
                continue

            handler = COMMAND_HANDLERS[token]
            index = handler.execute(commands, index)
            print_state(arg_count, commands, index)

        result = commands[-1] if commands else "No result"
        print("result:", result)

    except PostfixError as e:
        print(e)


if __name__ == "__main__":
    main()
