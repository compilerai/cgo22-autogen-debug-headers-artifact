#include "globals.h"

int s2712()
{

//	control flow
//	if to elemental min

		for (int i = 0; i < LEN; i++) {
			if (a[i] > b[i]) {
				a[i] += b[i] * c[i];
			}
		}
	return 0;
}

