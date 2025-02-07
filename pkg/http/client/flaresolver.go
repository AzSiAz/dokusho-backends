package client

type FlareSolverClient struct {
	URL string
}

func NewFlareSolverClient(url string) *FlareSolverClient {
	return &FlareSolverClient{
		URL: url,
	}
}

func (f *FlareSolverClient) Bypass(url string) ([]byte, error) {
	return nil, nil
}

func (f *FlareSolverClient) Ping() error {
	return nil
}
