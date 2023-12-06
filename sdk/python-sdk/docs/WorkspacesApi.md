# kraken_sdk.WorkspacesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_invitation**](WorkspacesApi.md#create_invitation) | **POST** /api/v1/workspaces/{uuid}/invitations | Invite a user to the workspace
[**create_workspace**](WorkspacesApi.md#create_workspace) | **POST** /api/v1/workspaces | Create a new workspace
[**delete_workspace**](WorkspacesApi.md#delete_workspace) | **DELETE** /api/v1/workspaces/{uuid} | Delete a workspace by its id
[**get_all_workspace_invitations**](WorkspacesApi.md#get_all_workspace_invitations) | **GET** /api/v1/workspaces/{uuid}/invitations | Query all open invitations to a workspace
[**get_all_workspaces**](WorkspacesApi.md#get_all_workspaces) | **GET** /api/v1/workspaces | Retrieve all workspaces that the executing user has access to
[**get_search_results**](WorkspacesApi.md#get_search_results) | **GET** /api/v1/workspaces/{w_uuid}/search/{s_uuid} | Retrieve results for a search by it&#39;s uuid
[**get_searches**](WorkspacesApi.md#get_searches) | **GET** /api/v1/workspaces/{uuid}/search | Query all searches
[**get_workspace**](WorkspacesApi.md#get_workspace) | **GET** /api/v1/workspaces/{uuid} | Retrieve a workspace by id
[**retract_invitation**](WorkspacesApi.md#retract_invitation) | **DELETE** /api/v1/workspaces/{w_uuid}/invitations/{i_uuid} | Retract an invitation to the workspace
[**search**](WorkspacesApi.md#search) | **POST** /api/v1/workspaces/{uuid}/search | Search through a workspaces&#39; data
[**transfer_ownership**](WorkspacesApi.md#transfer_ownership) | **POST** /api/v1/workspaces/{uuid}/transfer | Transfer ownership to another account
[**update_workspace**](WorkspacesApi.md#update_workspace) | **PUT** /api/v1/workspaces/{uuid} | Updates a workspace by its id


# **create_invitation**
> create_invitation(uuid, invite_to_workspace)

Invite a user to the workspace

Invite a user to the workspace  This action can only be invoked by the owner of a workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.invite_to_workspace import InviteToWorkspace
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 
    invite_to_workspace = kraken_sdk.InviteToWorkspace() # InviteToWorkspace | 

    try:
        # Invite a user to the workspace
        await api_instance.create_invitation(uuid, invite_to_workspace)
    except Exception as e:
        print("Exception when calling WorkspacesApi->create_invitation: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **invite_to_workspace** | [**InviteToWorkspace**](InviteToWorkspace.md)|  | 

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
**200** | The user was invited. |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_workspace**
> UuidResponse create_workspace(create_workspace_request)

Create a new workspace

Create a new workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_workspace_request import CreateWorkspaceRequest
from kraken_sdk.models.uuid_response import UuidResponse
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    create_workspace_request = kraken_sdk.CreateWorkspaceRequest() # CreateWorkspaceRequest | 

    try:
        # Create a new workspace
        api_response = await api_instance.create_workspace(create_workspace_request)
        print("The response of WorkspacesApi->create_workspace:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->create_workspace: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_workspace_request** | [**CreateWorkspaceRequest**](CreateWorkspaceRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Workspace was created |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_workspace**
> delete_workspace(uuid)

Delete a workspace by its id

Delete a workspace by its id

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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Delete a workspace by its id
        await api_instance.delete_workspace(uuid)
    except Exception as e:
        print("Exception when calling WorkspacesApi->delete_workspace: %s\n" % e)
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
**200** | Workspace was deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_workspace_invitations**
> WorkspaceInvitationList get_all_workspace_invitations(uuid)

Query all open invitations to a workspace

Query all open invitations to a workspace

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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Query all open invitations to a workspace
        api_response = await api_instance.get_all_workspace_invitations(uuid)
        print("The response of WorkspacesApi->get_all_workspace_invitations:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->get_all_workspace_invitations: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

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
**200** | Returns all open invitations to the workspace. |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_workspaces**
> GetAllWorkspacesResponse get_all_workspaces()

Retrieve all workspaces that the executing user has access to

Retrieve all workspaces that the executing user has access to  For administration access, look at the `/admin/workspaces` endpoint.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_workspaces_response import GetAllWorkspacesResponse
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)

    try:
        # Retrieve all workspaces that the executing user has access to
        api_response = await api_instance.get_all_workspaces()
        print("The response of WorkspacesApi->get_all_workspaces:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->get_all_workspaces: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllWorkspacesResponse**](GetAllWorkspacesResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns all workspaces that the executing user has access to |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_search_results**
> SearchResultPage get_search_results(w_uuid, s_uuid, limit, offset)

Retrieve results for a search by it's uuid

Retrieve results for a search by it's uuid

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.search_result_page import SearchResultPage
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    w_uuid = 'w_uuid_example' # str | The UUID of the workspace
    s_uuid = 's_uuid_example' # str | The UUID of the search
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve results for a search by it's uuid
        api_response = await api_instance.get_search_results(w_uuid, s_uuid, limit, offset)
        print("The response of WorkspacesApi->get_search_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->get_search_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**| The UUID of the workspace | 
 **s_uuid** | **str**| The UUID of the search | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**SearchResultPage**](SearchResultPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Search results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_searches**
> SearchesResultPage get_searches(uuid, limit, offset)

Query all searches

Query all searches

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.searches_result_page import SearchesResultPage
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Query all searches
        api_response = await api_instance.get_searches(uuid, limit, offset)
        print("The response of WorkspacesApi->get_searches:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->get_searches: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**SearchesResultPage**](SearchesResultPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Search results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_workspace**
> FullWorkspace get_workspace(uuid)

Retrieve a workspace by id

Retrieve a workspace by id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.full_workspace import FullWorkspace
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Retrieve a workspace by id
        api_response = await api_instance.get_workspace(uuid)
        print("The response of WorkspacesApi->get_workspace:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->get_workspace: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**FullWorkspace**](FullWorkspace.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns the workspace |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **retract_invitation**
> retract_invitation(w_uuid, i_uuid)

Retract an invitation to the workspace

Retract an invitation to the workspace  This action can only be invoked by the owner of a workspace

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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    w_uuid = 'w_uuid_example' # str | The UUID of the workspace
    i_uuid = 'i_uuid_example' # str | The UUID of the invitation

    try:
        # Retract an invitation to the workspace
        await api_instance.retract_invitation(w_uuid, i_uuid)
    except Exception as e:
        print("Exception when calling WorkspacesApi->retract_invitation: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**| The UUID of the workspace | 
 **i_uuid** | **str**| The UUID of the invitation | 

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
**200** | The invitation was retracted. |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **search**
> UuidResponse search(uuid, search_workspace_request)

Search through a workspaces' data

Search through a workspaces' data

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.search_workspace_request import SearchWorkspaceRequest
from kraken_sdk.models.uuid_response import UuidResponse
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 
    search_workspace_request = kraken_sdk.SearchWorkspaceRequest() # SearchWorkspaceRequest | 

    try:
        # Search through a workspaces' data
        api_response = await api_instance.search(uuid, search_workspace_request)
        print("The response of WorkspacesApi->search:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling WorkspacesApi->search: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **search_workspace_request** | [**SearchWorkspaceRequest**](SearchWorkspaceRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Search has been scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **transfer_ownership**
> transfer_ownership(uuid, transfer_workspace_request)

Transfer ownership to another account

Transfer ownership to another account  You will loose access to the workspace.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.transfer_workspace_request import TransferWorkspaceRequest
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 
    transfer_workspace_request = kraken_sdk.TransferWorkspaceRequest() # TransferWorkspaceRequest | 

    try:
        # Transfer ownership to another account
        await api_instance.transfer_ownership(uuid, transfer_workspace_request)
    except Exception as e:
        print("Exception when calling WorkspacesApi->transfer_ownership: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **transfer_workspace_request** | [**TransferWorkspaceRequest**](TransferWorkspaceRequest.md)|  | 

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
**200** | Workspace was transferred |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_workspace**
> update_workspace(uuid, update_workspace_request)

Updates a workspace by its id

Updates a workspace by its id  All parameter are optional, but at least one of them must be specified.  `name` must not be empty.  You can set `description` to null to remove the description from the database. If you leave the parameter out, the description will remain unchanged.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_workspace_request import UpdateWorkspaceRequest
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
    api_instance = kraken_sdk.WorkspacesApi(api_client)
    uuid = 'uuid_example' # str | 
    update_workspace_request = kraken_sdk.UpdateWorkspaceRequest() # UpdateWorkspaceRequest | 

    try:
        # Updates a workspace by its id
        await api_instance.update_workspace(uuid, update_workspace_request)
    except Exception as e:
        print("Exception when calling WorkspacesApi->update_workspace: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **update_workspace_request** | [**UpdateWorkspaceRequest**](UpdateWorkspaceRequest.md)|  | 

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
**200** | Workspace got updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

