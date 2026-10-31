#ifndef xd
#define xd e
#endif
#ifndef o7
#define o7 c
#endif
f float d9(c l,c b){float Og=dot(l,b);float yd=dot(l,l)*dot(b,b);return(yd==.0)?1.:clamp(Og*inversesqrt(yd),-1.,1.);}f void Pg(c C0,c G0,c M0,c V0,n1(c) E,n1(c) J,n1(c) D2){D2=G0-C0;c p7=M0-G0;c e9=V0-C0;J=p7-D2;E=-3.*p7+e9;}f X cb(c C0,c G0,c M0,c V0){X t;t[0]=(any(notEqual(C0,G0))?G0:any(notEqual(G0,M0))?M0:V0)-C0;t[1]=V0-(any(notEqual(V0,M0))?M0:any(notEqual(M0,G0))?G0:C0);return t;}f float Qg(c C0,c G0,c M0,c V0,float L1,float Rg){c E,J,D2;Pg(C0,G0,M0,V0,E,J,D2);c q7=3.*(((E*L1)+2.*J)*L1+D2);float zd=length(q7);if(zd==.0){return.0;}q7*=1./zd;float f9=2.*dot(E,q7);float r7=3.*(f9*L1+4.*dot(J,q7))*L1+6.*dot(D2,q7);float db=min(L1,1.-L1);float Sg=(f9*db*db+r7)*db;float Ad=min(Rg,Sg*.9999);float x3;if(f9==.0){x3=Ad/r7;}else{float U=1./f9;float b=r7*U,Y1=-Ad*U;float v7=(-1./3.)*b,w7=.5*Y1;float Bd=w7*w7-v7*v7*v7;if(Bd<.0){float g9=sqrt(v7);float h1=acos(w7/(g9*g9*g9));x3=-2.*g9*cos(h1*(1./3.)+(-z4*2./3.));}else{float E=pow(abs(w7)+sqrt(Bd),1./3.);if(w7<.0) E=-E;x3=E!=.0?E+v7/E:.0;}}x3=abs(x3);e t0011=L1+xd(-x3,-x3,x3,x3);e Cd=(E.xyxy*t0011+2.*J.xyxy)*t0011+D2.xyxy;X E2=cb(C0,G0,M0,V0);c Tg=t0011.x<1e-3?E2[0]:Cd.xy;c Ug=t0011.z>1.-1e-3?E2[1]:Cd.zw;return acos(d9(Tg,Ug));}f float h9(float l,float b){l=b<.0?-l:l;b=abs(b);return l>.0?(l<b?l/b:1.):.0;}float Vg(c C0,c G0,c M0,c V0,n1(float) eb){c Dd=V0-C0;float Ed=length(V0-C0);if(Ed==.0){eb=.5;return.0;}c M1=o7(-Dd.y,Dd.x)/Ed;float Fd=dot(M1,M0-C0);float f5=dot(M1,G0-C0);float g5=f5-Fd;
#if 0
float l=3.*g5;float Gd=g5+f5;float Y1=f5;float N2=sqrt(max(g5*g5+Fd*f5,.0));if(Gd<.0) N2=-N2;N2+=Gd;c x7=o7(h9(N2,l),h9(Y1,N2));c w6=3.*(x7*(x7*(x7*g5-(f5+g5))+f5));w6=abs(w6);eb=w6.x>w6.y?x7.x:x7.y;return max(w6.x,w6.y);
#else
float Hd=3.*g5;float J=-f5-g5;float D2=f5;float t=.5;for(int P0=0;P0<3;++P0){float Id=Hd*t;t=h9(Id*t-D2,2.*(Id+J));}eb=t;return abs(t*(t*(t*Hd+3.*J)+3.*D2));
#endif
}