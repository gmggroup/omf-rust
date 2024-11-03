from os import path
from unittest import TestCase

import numpy
import omf_python


class TestBlockModel(TestCase):
    def setUp(self) -> None:
        self.omf_dir = path.join(path.dirname(__file__), "data")
        self.one_of_everything = path.join(self.omf_dir, "one_of_everything.omf")

    def test_should_return_expected_regular_block_model(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model: omf_python.BlockModel = project.elements()[4].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        orientation = block_model.orient
        self.assertTrue(numpy.array_equal([-1, -1, -1], orientation.origin))
        self.assertTrue(numpy.array_equal([1, 0, 0], orientation.u))
        self.assertTrue(numpy.array_equal([0, 1, 0], orientation.v))
        self.assertTrue(numpy.array_equal([0, 0, 1], orientation.w))

        grid = block_model.grid
        self.assertIsInstance(grid, omf_python.Grid3Regular)
        self.assertEqual([1, 1, 1], grid.size)
        self.assertEqual([2, 2, 2], grid.count())
        self.assertEqual(8, grid.flat_count())
        self.assertEqual(27, grid.flat_corner_count())

        subblocks = block_model.subblocks
        self.assertIsNone(subblocks)

    def test_should_have_expected_tensor_block_model(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model: omf_python.BlockModel = project.elements()[5].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        grid = block_model.grid
        self.assertIsInstance(grid, omf_python.Grid3Tensor)
        self.assertEqual([2, 2, 2], grid.count())
        self.assertEqual(8, grid.flat_count())
        self.assertEqual(27, grid.flat_corner_count())

        u = grid.u
        self.assertIsInstance(u, omf_python.ScalarArray)
        self.assertEqual(2, u.item_count())
        u_scalars = reader.array_scalars(u)
        self.assertEqual(numpy.float64, u_scalars.dtype)
        self.assertTrue(numpy.array_equal([0.6666, 1.333], u_scalars))

        v = grid.v
        self.assertIsInstance(v, omf_python.ScalarArray)
        self.assertEqual(2, v.item_count())
        v_scalars = reader.array_scalars(v)
        self.assertEqual(numpy.float64, v_scalars.dtype)
        self.assertTrue(numpy.array_equal([0.6666, 1.333], v_scalars))

        w = grid.w
        self.assertIsInstance(w, omf_python.ScalarArray)
        self.assertEqual(2, w.item_count())
        w_scalars = reader.array_scalars(w)
        self.assertEqual(numpy.float64, w_scalars.dtype)
        self.assertTrue(numpy.array_equal([1.0, 1.0], w_scalars))

    def test_should_have_expected_regular_subblocks(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model: omf_python.BlockModel = project.elements()[6].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        regular_subblocks = block_model.subblocks
        self.assertIsInstance(regular_subblocks, omf_python.RegularSubblocks)

        expected_mode = omf_python.SubblockMode.Octree
        self.assertEqual(expected_mode, regular_subblocks.mode)

        expected_count = [4, 4, 4]
        self.assertEqual(expected_count, regular_subblocks.count)

        subblocks_array = regular_subblocks.subblocks
        self.assertIsInstance(subblocks_array, omf_python.RegularSubblockArray)

        expected_regular_subblock_parents = numpy.array(
            [
                [0, 0, 0],
                [1, 0, 0],
                [0, 1, 0],
                [1, 1, 0],
                [0, 0, 1],
                [1, 0, 1],
                [0, 1, 1],
                [1, 1, 1],
                [1, 1, 1],
                [1, 1, 1],
                [1, 1, 1],
            ]
        )
        expected_regular_subblock_corners = numpy.array(
            [
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 4, 4, 4],
                [0, 0, 0, 2, 2, 2],
                [0, 2, 0, 1, 3, 1],
                [2, 0, 0, 3, 1, 1],
                [0, 0, 2, 1, 1, 3],
            ]
        )

        regular_subblock_parents, regular_subblock_corners = (
            reader.array_regular_subblocks(subblocks_array)
        )
        self.assertTrue(
            numpy.array_equal(
                expected_regular_subblock_parents,
                regular_subblock_parents,
            )
        )
        self.assertTrue(
            numpy.array_equal(
                expected_regular_subblock_corners,
                regular_subblock_corners,
            )
        )

    def test_should_have_expected_freeform_subblocks(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model: omf_python.BlockModel = project.elements()[7].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        freeform_subblocks = block_model.subblocks
        self.assertIsInstance(freeform_subblocks, omf_python.FreeformSubblocks)

        subblocks_array = freeform_subblocks.subblocks
        self.assertIsInstance(subblocks_array, omf_python.FreeformSubblockArray)

        expected_freeform_subblock_parents = numpy.array(
            [
                [0, 0, 0],
                [1, 0, 0],
                [0, 1, 0],
                [1, 1, 0],
                [0, 0, 1],
                [1, 0, 1],
                [0, 1, 1],
                [1, 1, 1],
                [1, 1, 1],
                [1, 1, 1],
            ]
        )
        expected_freeform_subblock_corners = numpy.array(
            [
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0, 1.0, 0.3333],
                [0.0, 0.0, 0.3333, 0.75, 0.75, 0.6666],
                [0.0, 0.0, 0.6666, 0.5, 0.5, 1.0],
            ]
        )

        freeform_subblock_parents, freeform_subblock_corners = (
            reader.array_freeform_subblocks(subblocks_array)
        )
        self.assertEqual(numpy.uint32, freeform_subblock_parents.dtype)

        self.assertTrue(
            numpy.array_equal(
                expected_freeform_subblock_parents, freeform_subblock_parents
            )
        )
        self.assertTrue(
            numpy.allclose(
                expected_freeform_subblock_corners, freeform_subblock_corners
            )
        )
