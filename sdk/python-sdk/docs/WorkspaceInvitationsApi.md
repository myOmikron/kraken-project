# kraken_sdk.WorkspaceInvitationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accept_invitation**](WorkspaceInvitationsApi.md#accept_invitation) | **POST** /api/v1/invitations/{uuid}/accept | Accept an invitation to a workspace
[**decline_invitation**](WorkspaceInvitationsApi.md#decline_invitation) | **POST** /api/v1/invitations/{uuid}/decline | Decline an invitation to a workspace
[**get_all_invitations**](WorkspaceInvitationsApi.md#get_all_invitations) | **GET** /api/v1/invitations | Retrieve all open invitations to workspaces the currently logged-in user


# **accept_invitation**
> accept_invitation(uuid)

Accept an invitation to a workspace

Accept an invitation to a workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
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
    api_instance = kraken_sdk.WorkspaceInvitationsApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Accept an invitation to a workspace
        await api_instance.accept_invitation(uuid)
    except Exception as e:
        print("Exception when calling WorkspaceInvitationsApi->accept_invitation: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Accept an invitation |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **decline_invitation**
> decline_invitation(uuid)

Decline an invitation to a workspace

Decline an invitation to a workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
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
    api_instance = kraken_sdk.WorkspaceInvitationsApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Decline an invitation to a workspace
        await api_instance.decline_invitation(uuid)
    except Exception as e:
        print("Exception when calling WorkspaceInvitationsApi->decline_invitation: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Decline an invitation |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_invitations**
> WorkspaceInvitationList get_all_invitations()

Retrieve all open invitations to workspaces the currently logged-in user

Retrieve all open invitations to workspaces the currently logged-in user has retrieved

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.workspace_invitation_list import WorkspaceInvitationList
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
    api_instance = kraken_sdk.WorkspaceInvitationsApi(api_client)

    try:
        # Retrieve all open invitations to workspaces the currently logged-in user
        api_response = await api_instance.get_all_invitations()
        print("The response of WorkspaceInvitationsApi->get_all_invitations:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspaceInvitationsApi->get_all_invitations: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**WorkspaceInvitationList**](WorkspaceInvitationList.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns all invitations of a user |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

