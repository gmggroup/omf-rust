import omf_python
from os import path
from unittest import TestCase

class TestVectorAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_vector_attribute_instance(self) -> None:
        attribute = self.project.elements[1].attributes[1]

        self.assertIsInstance(attribute.get_data(), omf_python.AttributeDataVector)

    def test_should_return_vector_attribute_values(self) -> None:
        attribute_data = self.project.elements[1].attributes[1].get_data()

        expected_values = [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0], None]
        actual_values = self.reader.array_vectors(attribute_data.values)

        self.assertEqual(expected_values, actual_values)
