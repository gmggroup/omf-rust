import omf_python
from os import path
from unittest import TestCase

class TestTextAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

        # Get the "Pyramid Lines" element, and its "Strings" attribute.
        self.attribute = self.project.elements()[2].attributes()[0]

    def test_should_return_text_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf_python.AttributeDataText)

    def test_should_return_text_attribute_values_item_count(self) -> None:
        actual_count = self.attribute.get_data().values.item_count()
        self.assertEqual(actual_count, 8)

    def test_should_return_text_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = [None, None, None, None, "sw", "se", "ne", "nw"]
        actual_values = self.reader.array_text(attribute_data.values)

        self.assertEqual(expected_values, actual_values)
