from os import path
from unittest import TestCase

import numpy
import omf2


class TestGeometrySurface(TestCase):
    def setUp(self) -> None:
        self.examples_dir = path.join(path.dirname(__file__), "../../examples")

    def test_should_contain_surface_geometry(self) -> None:
        # Given the pyramid sample omf file
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf2.Reader(omf_file)
        project, _ = reader.project()

        # Geometry is an instance of omf2.Surface
        surface = project.elements()[0].geometry()
        self.assertIsInstance(surface, omf2.Surface)

        # And it contains 6 triangles
        triangles_array = surface.triangles
        self.assertEqual(6, triangles_array.item_count())

        # And it contains 5 vertices
        vertices_array = surface.vertices
        self.assertEqual(5, vertices_array.item_count())

        # And the vertices are what we expect
        vertices = reader.array_vertices(vertices_array)
        VERTICES = numpy.array(
            [
                [-1, -1, 0],
                [1, -1, 0],
                [1, 1, 0],
                [-1, 1, 0],
                [0, 0, 1],
            ]
        )
        self.assertEqual(numpy.float32, vertices.dtype)
        self.assertTrue(numpy.array_equal(VERTICES, vertices))

        # And the triangles are what we expect
        triangles = reader.array_triangles(triangles_array)
        TRIANGLES = numpy.array(
            [
                [0, 1, 4],
                [1, 2, 4],
                [2, 3, 4],
                [3, 0, 4],
                [0, 2, 1],
                [0, 3, 2],
            ]
        )
        self.assertTrue(numpy.array_equal(TRIANGLES, triangles))

    def test_should_contain_color(self) -> None:
        # Given
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf2.Reader(omf_file)
        project, _ = reader.project()

        surface = project.elements()[0]

        # Then
        self.assertEqual((255, 255, 0, 255), surface.color)
