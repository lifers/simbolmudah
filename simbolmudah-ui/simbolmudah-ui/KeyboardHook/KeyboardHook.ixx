#include "pch.h"
export module KeyboardHook;

using namespace winrt;
using namespace Windows::Foundation;

export class KeyboardHook
{
public:
	KeyboardHook() = default;
	explicit KeyboardHook(const delegate<DWORD>& reporterFn) { Reporter = reporterFn; }
	~KeyboardHook();
	IAsyncAction RunAndMonitorListeners();

private:
	static delegate<DWORD> Reporter;
	HHOOK m_hook{ nullptr };
	void Unregister();
	static LRESULT CALLBACK KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam);
};
