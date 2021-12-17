#include "globals.h"

int s125()
{

//	induction variable recognition
//	induction variable in two loops; collapsing possible

	int k;
		k = -1;
		for (int i = 0; i < LEN2; i++) {
			for (int j = 0; j < LEN2; j++) {
				k++;
				array[k] = aa[i][j] + bb[i][j] * cc[i][j];
			}
		}
	return 0;
}
