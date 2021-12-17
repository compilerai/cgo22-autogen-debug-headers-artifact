#include "globals.h"

int s162(int k)
{
//	control flow
//	deriving assertions

		if (k > 0) {
			for (int i = 0; i < LEN-1; i++) {
				a[i] = a[i + k] + b[i] * c[i];
			}
		}
	return 0;
}

