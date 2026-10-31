#ifndef ud
#define ud e
#endif
#ifndef o7
#define o7 c
#endif
f float Y8(c l,c b){float Gg=dot(l,b);float vd=dot(l,l)*dot(b,b);return(vd==.0)?1.:clamp(Gg*inversesqrt(vd),-1.,1.);}f void Hg(c A0,c F0,c L0,c S0,k1(c) B,k1(c) J,k1(c) B2){B2=F0-A0;c p7=L0-F0;c Z8=S0-A0;J=p7-B2;B=-3.*p7+Z8;}f X Za(c A0,c F0,c L0,c S0){X t;t[0]=(any(notEqual(A0,F0))?F0:any(notEqual(F0,L0))?L0:S0)-A0;t[1]=S0-(any(notEqual(S0,L0))?L0:any(notEqual(L0,F0))?F0:A0);return t;}f float Ig(c A0,c F0,c L0,c S0,float J1,float Jg){c B,J,B2;Hg(A0,F0,L0,S0,B,J,B2);c q7=3.*(((B*J1)+2.*J)*J1+B2);float wd=length(q7);if(wd==.0){return.0;}q7*=1./wd;float a9=2.*dot(B,q7);float r7=3.*(a9*J1+4.*dot(J,q7))*J1+6.*dot(B2,q7);float ab=min(J1,1.-J1);float Kg=(a9*ab*ab+r7)*ab;float xd=min(Jg,Kg*.9999);float q3;if(a9==.0){q3=xd/r7;}else{float U=1./a9;float b=r7*U,W1=-xd*U;float v7=(-1./3.)*b,w7=.5*W1;float yd=w7*w7-v7*v7*v7;if(yd<.0){float c9=sqrt(v7);float f1=acos(w7/(c9*c9*c9));q3=-2.*c9*cos(f1*(1./3.)+(-w4*2./3.));}else{float B=pow(abs(w7)+sqrt(yd),1./3.);if(w7<.0) B=-B;q3=B!=.0?B+v7/B:.0;}}q3=abs(q3);e t0011=J1+ud(-q3,-q3,q3,q3);e zd=(B.xyxy*t0011+2.*J.xyxy)*t0011+B2.xyxy;X C2=Za(A0,F0,L0,S0);c Lg=t0011.x<1e-3?C2[0]:zd.xy;c Mg=t0011.z>1.-1e-3?C2[1]:zd.zw;return acos(Y8(Lg,Mg));}f float d9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Ng(c A0,c F0,c L0,c S0,k1(float) bb){c Ad=S0-A0;float Bd=length(S0-A0);if(Bd==.0){bb=.5;return.0;}c K1=o7(-Ad.y,Ad.x)/Bd;float Cd=dot(K1,L0-A0);float Z4=dot(K1,F0-A0);float a5=Z4-Cd;
#if 0
float l=3.*a5;float Dd=a5+Z4;float W1=Z4;float L2=sqrt(max(a5*a5+Cd*Z4,.0));if(Dd<.0) L2=-L2;L2+=Dd;c x7=o7(d9(L2,l),d9(W1,L2));c q6=3.*(x7*(x7*(x7*a5-(Z4+a5))+Z4));q6=abs(q6);bb=q6.x>q6.y?x7.x:x7.y;return max(q6.x,q6.y);
#else
float Ed=3.*a5;float J=-Z4-a5;float B2=Z4;float t=.5;for(int P0=0;P0<3;++P0){float Fd=Ed*t;t=d9(Fd*t-B2,2.*(Fd+J));}bb=t;return abs(t*(t*(t*Ed+3.*J)+3.*B2));
#endif
}
