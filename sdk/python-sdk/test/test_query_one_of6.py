# coding: utf-8

"""
    kraken

    The core component of kraken-project

    The version of the OpenAPI document: 0.1.0
    Contact: git@omikron.dev
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest
import datetime

from kraken_sdk.models.query_one_of6 import QueryOneOf6

class TestQueryOneOf6(unittest.TestCase):
    """QueryOneOf6 unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> QueryOneOf6:
        """Test QueryOneOf6
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `QueryOneOf6`
        """
        model = QueryOneOf6()
        if include_optional:
            return QueryOneOf6(
                domain = None
            )
        else:
            return QueryOneOf6(
                domain = None,
        )
        """

    def testQueryOneOf6(self):
        """Test QueryOneOf6"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()