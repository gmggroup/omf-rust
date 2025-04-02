from os import path
from unittest import TestCase

import numpy
import omf2


class TestMappedTextureAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[9].attributes()[1]

        test_png = path.join(omf_dir, "test.png")
        with open(test_png, "rb") as file:
            self.image = file.read()

    def test_should_return_expected_texture_attributes(self) -> None:
        # And it should have texture coordinates
        texcoords = self.attribute.get_data().texcoords

        # And those coordinates should match the expected value
        self.assertEqual(4, texcoords.item_count())
        coordinates = self.reader.array_texcoords(texcoords)
        self.assertEqual(4, len(coordinates))
        self.assertEqual(numpy.float64, coordinates.dtype)
        COORDINATES = numpy.array([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        self.assertTrue(numpy.array_equal(COORDINATES, coordinates))

        # And it should contain an image
        image = self.attribute.get_data().image
        # Images always have an item count of zero
        self.assertEqual(0, image.item_count())

        # And the image contents should match those expected
        image_bytes = self.reader.image_bytes(image)
        self.assertEqual(self.image, image_bytes)
