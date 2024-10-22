%struct.Int_Int = type { i64 }

%struct.siko_Unit = type {  }

define void @siko_Unit(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Unit, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define void @Main_main(ptr noundef %fn_result) {
block0:
   %i_0_6 = alloca %struct.siko_Unit, align 4
   %i_0_5 = alloca %struct.siko_Unit, align 4
   %i_0_4 = alloca %struct.Int_Int, align 8
   %tmp_0_1 = alloca %struct.Int_Int, align 8
   %i_0_2 = alloca %struct.Int_Int, align 8
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 6, ptr %tmp_i_0_1_1, align 8
   %tmp_i_0_2_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_2, i32 0, i32 0
   store i64 5, ptr %tmp_i_0_2_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %tmp_0_1, ptr align 8 %i_0_2, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_0_4, ptr align 8 %tmp_0_1, i8 8, i1 false)
   call void @Int_Int_lessThan(ptr %i_0_4, ptr %i_0_1, ptr %i_0_5)
   call void @siko_Unit(ptr %i_0_6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_6, i8 0, i1 false)
   ret void
}

define void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result) {
}

define i32 @main() {
   %res = alloca %struct.siko_Unit, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


