import omf_python
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
        # TODO: currently raises pyo3_runtime.PanicException, should raise a sensible exception here
        pass
