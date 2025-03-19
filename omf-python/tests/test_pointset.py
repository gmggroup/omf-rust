from os import path
from unittest import TestCase

import numpy
import omf2


class TestPointSet(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.pointset = self.project.elements()[1]

    def test_should_return_expected_geometry_type(self) -> None:
        self.assertIsInstance(self.pointset.geometry(), omf2.PointSet)

    def test_should_return_expected_origin(self) -> None:
        pointset_origin = self.pointset.geometry().origin
        self.assertTrue(numpy.array_equal(pointset_origin, [0.0, 0.0, 0.0]))

    def test_should_return_expected_vertices(self) -> None:
        # Given
        vertices_array = self.pointset.geometry().vertices

        # When
        vertices = self.reader.array_vertices(vertices_array)

        # Then
        expected_vertices = numpy.array(
            [
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ]
        )
        self.assertEqual(numpy.float32, vertices.dtype)
        self.assertTrue(numpy.array_equal(vertices, expected_vertices))
