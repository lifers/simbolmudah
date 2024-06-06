#include "pch.h"
export module KeyboardHook;

using namespace winrt;
using namespace Windows::Foundation;

export class KeyboardHook
{
public:
	explicit KeyboardHook(const delegate<DWORD>& reporterFn) : m_handle(RunAndMonitorListeners())
	{
		Reporter = reporterFn;
	}
	~KeyboardHook();

private:
	static delegate<DWORD> Reporter;
	static LRESULT CALLBACK KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam);
	IAsyncAction RunAndMonitorListeners();
	const IAsyncAction m_handle;
	HHOOK m_hook{ nullptr };
};
