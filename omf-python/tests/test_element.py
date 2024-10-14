import omf_python
from os import path
from unittest import TestCase

class TestElement(TestCase):
    def test_should_return_expected_element_metadata(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")

        reader = omf_python.Reader(path.join(omf_dir, "element_metadata.omf"))
        project, _ = reader.project()

        element = project.elements()[0]

        expected_metadata = {
            "date_created": "2024-10-14T00:00:00Z",
            "date_modified": "2024-10-15T00:00:00Z",
            "sub-type": "point"
        }
        self.assertEqual(expected_metadata, element.metadata)
