# kraken_sdk.UserManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_users**](UserManagementApi.md#get_all_users) | **GET** /api/v1/users | Request all users
[**get_me**](UserManagementApi.md#get_me) | **GET** /api/v1/users/me | Retrieve the own user
[**set_password**](UserManagementApi.md#set_password) | **POST** /api/v1/users/setPassword | Set a new password
[**update_me**](UserManagementApi.md#update_me) | **PUT** /api/v1/users/me | Updates the own user


# **get_all_users**
> GetAllUsersResponse get_all_users()

Request all users

Request all users  This may be used to create invitations for workspaces

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_users_response import GetAllUsersResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.UserManagementApi(api_client)

    try:
        # Request all users
        api_response = await api_instance.get_all_users()
        print("The response of UserManagementApi->get_all_users:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UserManagementApi->get_all_users: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllUsersResponse**](GetAllUsersResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Simple representation of all users |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_me**
> GetUser get_me()

Retrieve the own user

Retrieve the own user

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_user import GetUser
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.UserManagementApi(api_client)

    try:
        # Retrieve the own user
        api_response = await api_instance.get_me()
        print("The response of UserManagementApi->get_me:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling UserManagementApi->get_me: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetUser**](GetUser.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns the own user |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **set_password**
> set_password(set_password_request)

Set a new password

Set a new password

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.set_password_request import SetPasswordRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.UserManagementApi(api_client)
    set_password_request = kraken_sdk.SetPasswordRequest() # SetPasswordRequest | 

    try:
        # Set a new password
        await api_instance.set_password(set_password_request)
    except Exception as e:
        print("Exception when calling UserManagementApi->set_password: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **set_password_request** | [**SetPasswordRequest**](SetPasswordRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Password was updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_me**
> update_me(update_me_request)

Updates the own user

Updates the own user  All parameters are optional, but at least one of them must be supplied.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_me_request import UpdateMeRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.UserManagementApi(api_client)
    update_me_request = kraken_sdk.UpdateMeRequest() # UpdateMeRequest | 

    try:
        # Updates the own user
        await api_instance.update_me(update_me_request)
    except Exception as e:
        print("Exception when calling UserManagementApi->update_me: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **update_me_request** | [**UpdateMeRequest**](UpdateMeRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Changes were applied |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

