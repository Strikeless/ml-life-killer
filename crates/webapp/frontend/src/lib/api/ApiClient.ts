import type { HttpBoardCreateRequest, HttpBoardCreateResponse, HttpBoardListRequest, HttpBoardListResponse } from "./types";

export class ApiClient {
    public url: URL;

    public static local(port: number = 11313): ApiClient {
        return new ApiClient(new URL(`http://localhost:${port}/api`));
    }

    public constructor(url: URL) {
        this.url = url;
    }

    public board_list = async (req: HttpBoardListRequest): Promise<HttpBoardListResponse> => await this.post("/board/list", req);
    public board_create = async (req: HttpBoardCreateRequest): Promise<HttpBoardCreateResponse> => await this.post("/board/create", req);

    private async get<Req, Resp>(endpoint: string, request_data: Req, method: string = "GET"): Promise<Resp> {
        if (request_data && method == "GET") {
            throw new Error("A GET request should not have a body. You may use a POST request instead, even if it is semantically incorrect.");
        }

        const endpointUrl = new URL(this.url.href + endpoint);
        const body = JSON.stringify(request_data);

        const response = await fetch(
            endpointUrl,
            {
                method,
                body,
                headers: {
                    "Content-Type": "application/json",
                    "Accept": "application/json",
                },
            },
        );
        const responseBodyText = await response.text();

        if (!response.ok) {
            throw new Error(`Non-OK API response of status ${response.status} ${response.statusText}: ${responseBodyText}`);
        }

        // If the server responded with malformed data, JSON.parse will throw a SyntaxError.
        const responseDeserialized = JSON.parse(responseBodyText);
        // If the server responded with JSON that doesn't match Resp, we'll catch that later on.
        return responseDeserialized as Resp;
    }

    private async post<Req, Resp>(endpoint: string, request_data: Req): Promise<Resp> {
        return await this.get(endpoint, request_data, "POST");
    }
}
