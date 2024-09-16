import omf_python
import pytest
from os import path
from unittest import TestCase


class TestReader(TestCase):
    def setUp(self) -> None:
        self.examples_dir = path.join(path.dirname(__file__), "../../examples")

    def test_should_return_expected_project_name(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf_python.Reader(omf_file)

        # Then
        self.assertEqual(reader.project.name, "Pyramid")

    def test_should_raise_expected_file_not_found_exception(self) -> None:
        # Given
        incorrect_location = path.join(self.examples_dir, "testfilenotfound.omf")

        # When
        with pytest.raises(OSError) as exception:
            reader = omf_python.Reader(incorrect_location)

        # Then
        self.assertEqual(str(exception.value), "No such file or directory (os error 2)")

