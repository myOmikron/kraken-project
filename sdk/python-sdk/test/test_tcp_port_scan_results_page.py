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

from kraken_sdk.models.tcp_port_scan_results_page import TcpPortScanResultsPage

class TestTcpPortScanResultsPage(unittest.TestCase):
    """TcpPortScanResultsPage unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> TcpPortScanResultsPage:
        """Test TcpPortScanResultsPage
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `TcpPortScanResultsPage`
        """
        model = TcpPortScanResultsPage()
        if include_optional:
            return TcpPortScanResultsPage(
                items = [
                    kraken_sdk.models.simple_tcp_port_scan_result.SimpleTcpPortScanResult(
                        uuid = '', 
                        attack = '', 
                        created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        address = '127.0.0.1', 
                        port = 0, )
                    ],
                limit = 50,
                offset = 0,
                total = 0
            )
        else:
            return TcpPortScanResultsPage(
                items = [
                    kraken_sdk.models.simple_tcp_port_scan_result.SimpleTcpPortScanResult(
                        uuid = '', 
                        attack = '', 
                        created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'), 
                        address = '127.0.0.1', 
                        port = 0, )
                    ],
                limit = 50,
                offset = 0,
                total = 0,
        )
        """

    def testTcpPortScanResultsPage(self):
        """Test TcpPortScanResultsPage"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()