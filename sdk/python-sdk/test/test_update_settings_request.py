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

from kraken_sdk.models.update_settings_request import UpdateSettingsRequest

class TestUpdateSettingsRequest(unittest.TestCase):
    """UpdateSettingsRequest unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> UpdateSettingsRequest:
        """Test UpdateSettingsRequest
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `UpdateSettingsRequest`
        """
        model = UpdateSettingsRequest()
        if include_optional:
            return UpdateSettingsRequest(
                mfa_required = True,
                oidc_initial_permission_level = 'ReadOnly',
                dehashed_email = 'foo@example.com',
                dehashed_api_key = '1231kb3kkb51kj31kjb231kj3b1jk23bkj123'
            )
        else:
            return UpdateSettingsRequest(
                mfa_required = True,
                oidc_initial_permission_level = 'ReadOnly',
        )
        """

    def testUpdateSettingsRequest(self):
        """Test UpdateSettingsRequest"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()