import omf_python
from os import path
from unittest import TestCase


class TestColorAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_expected_color_attributes(self) -> None:
        attributes = self.project.elements[0].attributes
        self.assertEqual(3, len(attributes))

        color_attributes = [a for a in attributes if a.name == "Colors"]
        self.assertEqual(1, len(color_attributes))

        values = color_attributes[0].get_data().values
        self.assertEqual(6, values.item_count)

        color_array = self.reader.array_color(values)
        self.assertEqual(6, len(color_array))
        COLORS = [
            omf_python.Color(255, 0, 0, 255),
            omf_python.Color(255, 255, 0, 255),
            omf_python.Color(0, 255, 0, 255),
            omf_python.Color(0, 0, 255, 255),
            omf_python.Color(255, 255, 255, 255),
            omf_python.Color(255, 255, 255, 255),
        ]
        self.assertEqual(COLORS, color_array)
