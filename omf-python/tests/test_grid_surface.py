from os import path
from unittest import TestCase

import numpy
import omf_python


class TestGridSurface(TestCase):
    def setUp(self) -> None:
        self.omf_dir = path.join(path.dirname(__file__), "data")
        self.one_of_everything = path.join(self.omf_dir, "one_of_everything.omf")

    def test_should_contain_grid_surface_geometry(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        grid_surface = project.elements()[3].geometry()
        self.assertIsInstance(grid_surface, omf_python.GridSurface)

        orientation = grid_surface.orient
        self.assertTrue(numpy.array_equal([-1.5, -1.5, 0], orientation.origin))
        self.assertTrue(numpy.array_equal([1, 0, 0], orientation.u))
        self.assertTrue(numpy.array_equal([0, 1, 0], orientation.v))

        height_array = grid_surface.heights
        self.assertEqual(9, height_array.item_count())
        height_scalars = reader.array_scalars(height_array)
        self.assertEqual(numpy.float32, height_scalars.dtype)
        self.assertTrue(
            numpy.array_equal(
                [0, 0, 0, 0, 2, 0, 0, 0, 0],
                height_scalars,
            )
        )

        grid = grid_surface.grid
        self.assertIsInstance(grid, omf_python.Grid2Tensor)
        self.assertEqual([2, 2], grid.count())

        u = grid.u
        self.assertIsInstance(u, omf_python.ScalarArray)
        self.assertEqual(2, u.item_count())
        u_scalars = reader.array_scalars(u)
        self.assertEqual(numpy.float64, u_scalars.dtype)
        self.assertTrue(numpy.array_equal([1, 2], u_scalars))

        v = grid.v
        self.assertIsInstance(v, omf_python.ScalarArray)
        self.assertEqual(2, v.item_count())
        v_scalars = reader.array_scalars(v)
        self.assertEqual(numpy.float64, v_scalars.dtype)
        self.assertTrue(numpy.array_equal([1.5, 1.5], v_scalars))
