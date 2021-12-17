#include "globals.h"

int s252()
{

//	scalar and array expansion
//	loop with ambiguous scalar temporary

	TYPE t, s;
		t = (TYPE) 0.;
		for (int i = 0; i < LEN; i++) {
			s = b[i] * c[i];
			a[i] = s + t;
			t = s;
		}
	return 0;
}

