from os import path
from unittest import TestCase

import omf_python


class TestVectorAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[1].attributes()[1]

    def test_should_return_vector_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf_python.AttributeDataVector)

    def test_should_return_vector_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            None,
        ]
        actual_values = self.reader.array_vectors(attribute_data.values)

        self.assertEqual(expected_values, actual_values)
