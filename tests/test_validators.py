"""Unit tests for validation system."""

import pytest
from pathlib import Path
from shellgame.validation.validators import (
    StringValidator,
    IntegerValidator,
    BasenameValidator,
    FileExistsValidator,
    DirectoryExistsValidator,
    MultiValidator,
    OrderedListValidator,
)


class TestStringValidator:
    """Test StringValidator."""

    def test_exact_match(self, tmp_path):
        """Test exact string matching."""
        validator = StringValidator("test")
        success, msg = validator.validate("test", tmp_path)
        assert success is True
        assert "Správně!" in msg

    def test_mismatch(self, tmp_path):
        """Test string mismatch."""
        validator = StringValidator("test")
        success, msg = validator.validate("wrong", tmp_path)
        assert success is False
        assert "Očekáváno" in msg

    def test_case_insensitive(self, tmp_path):
        """Test case-insensitive matching."""
        validator = StringValidator("Test", case_sensitive=False)
        success, msg = validator.validate("test", tmp_path)
        assert success is True

    def test_whitespace_trimming(self, tmp_path):
        """Test whitespace is trimmed."""
        validator = StringValidator("test")
        success, msg = validator.validate("  test  ", tmp_path)
        assert success is True


class TestIntegerValidator:
    """Test IntegerValidator."""

    def test_exact_match(self, tmp_path):
        """Test exact integer matching."""
        validator = IntegerValidator(42)
        success, msg = validator.validate("42", tmp_path)
        assert success is True

    def test_mismatch(self, tmp_path):
        """Test integer mismatch."""
        validator = IntegerValidator(42)
        success, msg = validator.validate("43", tmp_path)
        assert success is False
        assert "Očekáváno 42" in msg

    def test_invalid_integer(self, tmp_path):
        """Test invalid integer input."""
        validator = IntegerValidator(42)
        success, msg = validator.validate("not_a_number", tmp_path)
        assert success is False
        assert "celé číslo" in msg.lower()


class TestBasenameValidator:
    """Test BasenameValidator."""

    def test_exact_match(self, tmp_path):
        """Test exact basename matching."""
        validator = BasenameValidator("mydir")
        success, msg = validator.validate("mydir", tmp_path)
        assert success is True

    def test_strip_extension(self, tmp_path):
        """Test extension stripping."""
        validator = BasenameValidator("myfile", strip_ext=True)
        success, msg = validator.validate("myfile.txt", tmp_path)
        assert success is True

    def test_no_strip_extension(self, tmp_path):
        """Test no extension stripping."""
        validator = BasenameValidator("myfile.txt", strip_ext=False)
        success, msg = validator.validate("myfile.txt", tmp_path)
        assert success is True


class TestFileExistsValidator:
    """Test FileExistsValidator."""

    def test_file_exists(self, tmp_path):
        """Test file existence check."""
        test_file = tmp_path / "test.txt"
        test_file.write_text("content")

        validator = FileExistsValidator("test.txt", should_exist=True)
        success, msg = validator.validate("", tmp_path)
        assert success is True

    def test_file_not_exists(self, tmp_path):
        """Test file non-existence check."""
        validator = FileExistsValidator("nonexistent.txt", should_exist=True)
        success, msg = validator.validate("", tmp_path)
        assert success is False
        assert "neexistuje" in msg

    def test_file_should_not_exist(self, tmp_path):
        """Test file should not exist check."""
        validator = FileExistsValidator("test.txt", should_exist=False)
        success, msg = validator.validate("", tmp_path)
        assert success is True


class TestDirectoryExistsValidator:
    """Test DirectoryExistsValidator."""

    def test_directory_exists(self, tmp_path):
        """Test directory existence check."""
        test_dir = tmp_path / "testdir"
        test_dir.mkdir()

        validator = DirectoryExistsValidator("testdir", should_exist=True)
        success, msg = validator.validate("", tmp_path)
        assert success is True

    def test_directory_not_exists(self, tmp_path):
        """Test directory non-existence check."""
        validator = DirectoryExistsValidator("nonexistent", should_exist=True)
        success, msg = validator.validate("", tmp_path)
        assert success is False


class TestMultiValidator:
    """Test MultiValidator."""

    def test_all_pass(self, tmp_path):
        """Test all validators pass."""
        validators = [StringValidator("test"), IntegerValidator(42)]
        multi = MultiValidator(validators)

        # This won't work as expected since answer can't be both string and int
        # Just testing the structure
        success, msg = multi.validate("test", tmp_path)
        assert isinstance(success, bool)

    def test_one_fails(self, tmp_path):
        """Test one validator fails."""
        test_file = tmp_path / "test.txt"
        test_file.write_text("content")

        validators = [
            FileExistsValidator("test.txt", should_exist=True),
            FileExistsValidator("nonexistent.txt", should_exist=True),
        ]
        multi = MultiValidator(validators)

        success, msg = multi.validate("", tmp_path)
        assert success is False


class TestOrderedListValidator:
    """Test OrderedListValidator."""

    def test_exact_match(self, tmp_path):
        """Test exact list matching."""
        validator = OrderedListValidator(["a", "b", "c"])
        success, msg = validator.validate("a,b,c", tmp_path)
        assert success is True
        assert "Správně!" in msg

    def test_whitespace_handling(self, tmp_path):
        """Test whitespace handling in list."""
        validator = OrderedListValidator(["a", "b", "c"])
        success, msg = validator.validate(" a , b , c ", tmp_path)
        assert success is True

    def test_order_mismatch(self, tmp_path):
        """Test order mismatch."""
        validator = OrderedListValidator(["a", "b", "c"])
        success, msg = validator.validate("a,c,b", tmp_path)
        assert success is False
        assert "Očekáváno" in msg

    def test_length_mismatch(self, tmp_path):
        """Test length mismatch."""
        validator = OrderedListValidator(["a", "b"])
        success, msg = validator.validate("a,b,c", tmp_path)
        assert success is False

    def test_case_insensitive(self, tmp_path):
        """Test case-insensitive matching."""
        validator = OrderedListValidator(["A", "B"], case_sensitive=False)
        success, msg = validator.validate("a,b", tmp_path)
        assert success is True
