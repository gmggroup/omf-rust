from os import path
from unittest import TestCase

import numpy
import omf_python


class TestProject(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "../../examples")
        pyramid = path.join(omf_dir, "pyramid/pyramid.omf")
        self.reader = omf_python.Reader(pyramid)

    def test_should_return_expected_project_details(self) -> None:
        # Given I have loaded a project
        project, _ = self.reader.project()

        # Then details should match what is in the project
        self.assertEqual(project.name, "Pyramid")
        self.assertEqual(project.description, "Contains a square pyramid.")
        self.assertEqual(project.coordinate_reference_system, "")
        self.assertEqual(project.units, "")
        self.assertTrue(numpy.array_equal(project.origin, [0.0, 0.0, 0.0]))
        self.assertEqual(project.author, "Somebody")
        self.assertEqual(project.application, "OMF Rust example")

    def test_should_return_elements(self) -> None:
        # Given I have loaded a project
        project, _ = self.reader.project()

        # When I get the elements
        elements = project.elements()

        # Then I should have two elements
        self.assertEqual(len(elements), 2)

        # And those elements should be of type omf_python.Element
        for element in elements:
            self.assertIsInstance(element, omf_python.Element)
