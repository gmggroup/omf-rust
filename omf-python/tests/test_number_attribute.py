import omf_python
from os import path
from unittest import TestCase

class TestNumberAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_number_attribute_instance(self) -> None:
        attribute = self.project.elements[0].attributes[2]

        self.assertIsInstance(attribute.get_data(), omf_python.AttributeDataNumber)

    def test_should_return_number_attribute_values(self) -> None:
        attribute_data = self.project.elements[0].attributes[2].get_data()

        expected_values = [946684800.0, 946688400.0, 946692000.0, 946695600.0, 946699200.0]
        actual_values = self.reader.array_numbers(attribute_data.values)

        self.assertEqual(expected_values, actual_values)
