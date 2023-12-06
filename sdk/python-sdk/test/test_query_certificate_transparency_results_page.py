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

from kraken_sdk.models.query_certificate_transparency_results_page import QueryCertificateTransparencyResultsPage

class TestQueryCertificateTransparencyResultsPage(unittest.TestCase):
    """QueryCertificateTransparencyResultsPage unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> QueryCertificateTransparencyResultsPage:
        """Test QueryCertificateTransparencyResultsPage
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `QueryCertificateTransparencyResultsPage`
        """
        model = QueryCertificateTransparencyResultsPage()
        if include_optional:
            return QueryCertificateTransparencyResultsPage(
                items = [
                    kraken_sdk.models.full_query_certificate_transparency_result.FullQueryCertificateTransparencyResult(
                        uuid = '', 
                        attack = '', 
                        created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        issuer_name = '', 
                        common_name = '', 
                        value_names = [
                            ''
                            ], 
                        not_before = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        not_after = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        serial_number = '', )
                    ],
                limit = 50,
                offset = 0,
                total = 0
            )
        else:
            return QueryCertificateTransparencyResultsPage(
                items = [
                    kraken_sdk.models.full_query_certificate_transparency_result.FullQueryCertificateTransparencyResult(
                        uuid = '', 
                        attack = '', 
                        created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        issuer_name = '', 
                        common_name = '', 
                        value_names = [
                            ''
                            ], 
                        not_before = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        not_after = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        serial_number = '', )
                    ],
                limit = 50,
                offset = 0,
                total = 0,
        )
        """

    def testQueryCertificateTransparencyResultsPage(self):
        """Test QueryCertificateTransparencyResultsPage"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()