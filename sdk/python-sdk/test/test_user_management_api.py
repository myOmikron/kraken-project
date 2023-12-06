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

from kraken_sdk.api.user_management_api import UserManagementApi


class TestUserManagementApi(unittest.TestCase):
    """UserManagementApi unit test stubs"""

    def setUp(self) -> None:
        self.api = UserManagementApi()

    def tearDown(self) -> None:
        pass

    def test_get_all_users(self) -> None:
        """Test case for get_all_users

        Request all users
        """
        pass

    def test_get_me(self) -> None:
        """Test case for get_me

        Retrieve the own user
        """
        pass

    def test_set_password(self) -> None:
        """Test case for set_password

        Set a new password
        """
        pass

    def test_update_me(self) -> None:
        """Test case for update_me

        Updates the own user
        """
        pass


if __name__ == '__main__':
    unittest.main()